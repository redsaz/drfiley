use std::future;
use std::option::Option;
use std::path::PathBuf;
use std::pin::Pin;

use api::agent_listen_response::{ChangeScanPaths, Heartbeat, What};
use futures::{Future, Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

use api::dr_filey_handler_server::{DrFileyHandler, DrFileyHandlerServer};
use api::{
    AgentListenResponse, AgentReadyRequest, AgentReadyResponse, Paths, ScanPaths, ScannedItem,
};

pub mod api {
    tonic::include_proto!("api");
}

pub struct DrFileyHandlerImpl {}

#[tonic::async_trait]
impl DrFileyHandler for DrFileyHandlerImpl {
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
    ) -> Result<Response<<DrFileyHandlerImpl as DrFileyHandler>::AgentListenStream>, Status> {
        // the stream is the communication back to the client.
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            // First, add a directory to periodically sync.
            // Normally this would happen at some point by the user going through the gui, but
            // since we're testing comms, for now we'll just have the server add the directory.
            let alr = AgentListenResponse {
                what: Some(What::ChangeScanPaths(ChangeScanPaths {
                    paths_to_add: vec![".".to_string()],
                    paths_to_delete: vec![],
                })),
            };
            if let Err(err) = tx.send(Ok(alr)) {
                println!("ERROR: failed to update stream client: {:?}", err);
                return;
            }

            loop {
                // We're still just testing things. Send a heartbeat periodically.
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                let alr = AgentListenResponse {
                    what: Some(What::Heartbeat(Heartbeat {})),
                };

                if let Err(err) = tx.send(Ok(alr)) {
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

impl Default for DrFileyHandlerImpl {
    fn default() -> Self {
        DrFileyHandlerImpl {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8888".parse()?;
    let handler = DrFileyHandlerImpl::default();

    println!("Server listening on {}", addr);

    // let reflection_service = tonic_reflection::server::Builder::configure()
    // .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
    // .build()
    // .unwrap();

    Server::builder()
        .add_service(DrFileyHandlerServer::new(handler))
        // .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
