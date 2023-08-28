use api::dr_filey_service_client::DrFileyServiceClient;
use api::{DrFileyRequest, FileStat};
use drfiley;
use futures::stream;
use rusqlite::{params, Connection, Result};

mod configuration;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = configuration::config().expect("DrFiley Agent may be misconfigured.");
    let path = &config.path;

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

    eprintln!("Scanning {}", path.display());
    let job = drfiley::jobs::ScanJob::new(path);
    let files = job.run();
    let mut conn = Connection::open("test-cache.sqlite")?;
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

    tx.commit()?;

    Ok(())
}
