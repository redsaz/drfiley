// pub mod api {
//     tonic::include_proto!("api");
// }

// use std::{str, thread};
// mod configuration;
// use tonic::{transport::Server, Request, Response, Status};
// use api::echo_service_server::{EchoService, EchoServiceServer};
// use api::{EchoRequest, EchoResponse};

// #[derive(Debug, Default)]
// pub struct Echo {}

// impl EchoService for Echo {
//     fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
//         println!("Got a request: {:?}", request);

//         let reply = EchoResponse {
//             message: format!("{}", request.into_inner().message),
//         };

//         Ok(Response::new(reply))
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = format!("{}:{}", "127.0.0.1", "50052").parse()?;
//     let echo = Echo::default();

//     println!("Server listening on {}", addr);

//     Server::builder()
//         .add_service(EchoServiceServer::new(echo))
//         .serve(addr)
//         .await?;

//     Ok(())
// }

// // fn main() {
// //     let config = configuration::config().expect("DrFiley Server must be configured.");

// //     let thread_one = thread::spawn(|| listen());

// //     thread_one.join().unwrap();
// // }

// fn listen() {
//     let ctx = zmq::Context::new();

//     let socket = ctx.socket(zmq::ROUTER).unwrap();
//     socket.bind("tcp://*:8888").unwrap();
//     let id = [125u8];
//     socket.set_identity(&id).expect("Identity failed to set.");
//     socket.set_router_mandatory(true).expect("router mandatory");
//     loop {
//         let id = socket.recv_bytes(0).expect("Getting id");
//         let data = socket.recv_string(0).expect("Testing recv");
//         println!(
//             "Id: {:?} Message: {}",
//             id,
//             data.expect("Error converting message to utf8")
//         );
//         socket.send(&id, zmq::SNDMORE).expect("testing id");
//         socket.send("", zmq::SNDMORE).expect("blank");
//         socket.send("Hello there", 0).expect("Testing reply");

//         socket.send(&id, zmq::SNDMORE).expect("testing id");
//         socket.send("", zmq::SNDMORE).expect("blank");
//         socket.send("Hello there again", 0).expect("Testing reply")
//     }
// }


use tonic::{transport::Server, Request, Response, Status};

use api::dr_filey_service_server::{DrFileyService, DrFileyServiceServer};
use api::{DrFileyRequest, DrFileyResponse};

pub mod api {
    tonic::include_proto!("api");
}

#[derive(Debug, Default)]
pub struct Echo {}

#[tonic::async_trait]
impl DrFileyService for Echo {
    async fn echo(&self, request: Request<DrFileyRequest>) -> Result<Response<DrFileyResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = DrFileyResponse {
            message: format!("{}", request.into_inner().message),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8888".parse()?;
    let echo = Echo::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(DrFileyServiceServer::new(echo))
        .serve(addr)
        .await?;

    Ok(())
}
