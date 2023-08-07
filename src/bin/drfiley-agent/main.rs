use api::dr_filey_service_client::DrFileyServiceClient;
use api::{DrFileyRequest, FileStat};
use drfiley;
use futures::stream;

mod configuration;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = configuration::config().expect("DrFiley Agent must be configured.");
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

    eprintln!("Stat all files in {}", path.display());
    let mut job = drfiley::jobs::StatJob::new(path);
    job.run();
    Ok(())
}
