use std::path::{Path, PathBuf};

use api::agent_listen_response::{ChangeScanPaths, What};
use api::dr_filey_handler_client::DrFileyHandlerClient;
use api::{AgentListenResponse, AgentReadyRequest};
use configuration::Configuration;
use drfiley;
use futures::stream;
use rusqlite::{params, Connection, Result};
use tonic::transport::Channel;
use tonic::Request;

use crate::api::{ScanPaths, ScannedItem};

mod configuration;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = configuration::config().expect("DrFiley Agent may be misconfigured.");

    let mut client = DrFileyHandlerClient::connect("http://127.0.0.1:8888").await?;

    let request = Request::new(AgentReadyRequest {
        key: Some("woo".to_string()),
    });
    let response = client.agent_ready(request).await?;

    let started_response = response.into_inner();
    if let Some(updated_key) = started_response.key {
        eprintln!("Updated key={:?}", updated_key);
    }

    let response = client.agent_listen(Request::new(())).await?;
    let mut listen_stream = response.into_inner();

    loop {
        let msg_opt = listen_stream.message().await?;
        match msg_opt {
            None => break,
            Some(msg) => {
                if let Some(what) = msg.what {
                    match what {
                        What::Heartbeat(_) => eprintln!("Got heartbeat."),
                        What::GetDirs(_) => todo!(),
                        What::GetScanPaths(_) => todo!(),
                        What::ChangeScanPaths(scan_paths) => {
                            change_scan_paths(&config, &scan_paths, &mut client).await?
                        }
                    }
                }
            }
        }
    }

    // START - do not connect to server yet
    // let mut client = DrFileyServiceClient::connect("http://127.0.0.1:8888").await?;

    // let request = tonic::Request::new(DrFileyRequest {
    //     message: "Test Message".to_string(),
    // });

    // let response = client.echo(request).await?;

    // println!("RESPONSE={:?}", response.into_inner().message);
    // let request = tonic::Request::new(
    //     stream::iter(drfiley::walker::walk(path)?.into_iter().map(
    //     |item| FileStat {
    //         path: item.unwrap().path().to_string_lossy().to_string(),
    //     },
    // )));

    // let response = client.file_stats(request).await?;
    // println!("RESPONSE={:?}", response.into_inner().message);
    // END - do not connect to server yet

    Ok(())
}

async fn change_scan_paths(
    config: &Configuration,
    scan_paths: &ChangeScanPaths,
    client: &mut DrFileyHandlerClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: This would be added to the config and a job would eventually start.
    // Here, we're short circuiting all of that in the name of testing.
    for path in scan_paths.paths_to_add.as_slice() {
        let path = PathBuf::from(path);
        scan_files(config, &path, client).await?;
    }
    Ok(())
}

async fn scan_files(
    config: &Configuration,
    path: &PathBuf,
    client: &mut DrFileyHandlerClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Scanning {}", path.display());
    let job = drfiley::jobs::ScanJob::new(path);
    let files = job.run();
    let mut conn = Connection::open("test-drfiley-cache.sqlite")?;
    conn.pragma_update_and_check(Option::None, "journal_mode", "WAL", |_row| Ok(()))?;
    conn.pragma_update(Option::None, "synchronous", "NORMAL")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL,
        numbytes INTEGER
    ) STRICT",
        (),
    )?;
    let tx = conn.transaction()?;

    files.iter().for_each(|file| {
        tx.execute(
            "INSERT INTO file (path, numbytes) VALUES (?1, ?2)",
            params!(file.path.to_str(), file.size_bytes),
        )
        .expect("TODO Don't panic");
    });

    let request = tonic::Request::new(stream::iter(files.into_iter().map(|item| ScannedItem {
        path: item.path.to_string_lossy().to_string(),
        size_bytes: item.size_bytes,
    })));

    let response = client.agent_scanned_items(request).await;
    if let Err(e) = response {
        eprintln!("RESPONSE ERROR={:?}", e);
        return Err(Box::new(e));
    }

    tx.commit()?;

    Ok(())
}
