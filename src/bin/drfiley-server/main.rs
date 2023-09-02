use std::future;
use std::option::Option;
use std::pin::Pin;

use api::agent_listen_response::{Heartbeat, What};
use futures::{Future, Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

use api::handler_server::{Handler, HandlerServer};
use api::{
    AgentListenResponse, AgentReadyRequest, AgentReadyResponse, Paths, ScanPaths, ScannedItem,
};

pub mod api {
    tonic::include_proto!("api");
}

pub struct DrFileyHandler {}

#[tonic::async_trait]
impl Handler for DrFileyHandler {
    type AgentListenStream =
        Pin<Box<dyn Stream<Item = Result<AgentListenResponse, Status>> + Send>>;

    async fn agent_ready(
        &self,
        request: Request<AgentReadyRequest>,
    ) -> Result<Response<AgentReadyResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = AgentReadyResponse { key: Option::None };

        Ok(Response::new(reply))
    }

    async fn agent_listen(
        &self,
        request: Request<()>,
    ) -> Result<Response<<DrFileyHandler as Handler>::AgentListenStream>, Status> {
        // the stream is the communication back to the client.
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                // For now, do something every 10 seconds
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                let aim = AgentListenResponse {
                    what: Some(What::Heartbeat(Heartbeat {})),
                };

                if let Err(err) = tx.send(Ok(aim)) {
                    println!("ERROR: failed to update stream client: {:?}", err);
                    return;
                }
            }
        });

        let stream = UnboundedReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::AgentListenStream))
    }

    async fn agent_scanned_items(
        &self,
        request: Request<Streaming<ScannedItem>>,
    ) -> Result<Response<()>, Status> {
        let mut count = 0;
        let fut = request.into_inner().for_each(|result| {
            count += 1;
            println!("Received: {:?}", result.unwrap().path);
            future::ready(())
        });
        fut.await;
        Ok(Response::new(()))
    }

    async fn reply_get_dirs(&self, _: Request<Paths>) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn reply_get_scan_paths(&self, _: Request<ScanPaths>) -> Result<Response<()>, Status> {
        todo!()
    }
}

impl Default for DrFileyHandler {
    fn default() -> Self {
        DrFileyHandler {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8888".parse()?;
    let handler = DrFileyHandler::default();

    println!("Server listening on {}", addr);

    // let reflection_service = tonic_reflection::server::Builder::configure()
    // .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
    // .build()
    // .unwrap();

    Server::builder()
        .add_service(HandlerServer::new(handler))
        // .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
