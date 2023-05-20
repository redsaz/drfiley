use drfiley;
use api::dr_filey_service_client::DrFileyServiceClient;
use api::DrFileyRequest;

mod configuration;

pub mod api {
    tonic::include_proto!("api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = configuration::config().expect("DrFiley Agent must be configured.");

    if let Err(_) = drfiley::stat_all(&config.path) {
        eprintln!("Error running DrFiley Agent.")
    }

    let mut client = DrFileyServiceClient::connect("http://127.0.0.1:8888").await?;

    let request = tonic::Request::new(DrFileyRequest {
        message: "Test Message".to_string(),
    });

    let response = client.echo(request).await?;

    println!("RESPONSE={:?}", response.into_inner().message);

    Ok(())
}
