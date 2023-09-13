use std::io::Error;
use std::option::Option;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use actix_web::rt::time::interval;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_lab::extract::Path;
use actix_web_lab::sse::{self, ChannelStream, Sse};
use actix_web_static_files::ResourceFiles;
use api::agent_listen_response::{ChangeScanPaths, Heartbeat, What};
use futures::future;
use futures::{Future, Stream, StreamExt};
use parking_lot::Mutex;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
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

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub struct DrFileyHandlerImpl {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

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
            let file = result.unwrap();
            println!("Received: {:?}", file.path);
            let mut conn = self.pool.get().expect("Couldn't get DB connection.");
            let tx = conn.transaction().expect("Couldn't start transaction.");
            tx.execute(
                "INSERT INTO file (path, numbytes) VALUES (?1, ?2)",
                params!(file.path.as_str(), file.size_bytes),
            )
            .expect("TODO Don't panic");
            tx.commit().expect("Couldn't commit transaction.");
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

pub struct AppState {
    broadcaster: Arc<Broadcaster>,
}

#[derive(Debug, Clone, Default)]
struct BroadcasterInner {
    clients: Vec<sse::Sender>,
}

pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

impl Broadcaster {
    /// Constructs new broadcaster and spawns ping loop.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Broadcaster {
            inner: Mutex::new(BroadcasterInner::default()),
        });
        Broadcaster::spawn_ping(Arc::clone(&this));

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast list if not.
    fn spawn_ping(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().clients.clone();
        println!("active client {:?}", clients);

        let mut ok_clients = Vec::new();

        println!("okay active client {:?}", ok_clients);

        for client in clients {
            if client
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_clients.push(client.clone());
            }
        }

        self.inner.lock().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self) -> Sse<ChannelStream> {
        println!("starting creation");
        let (tx, rx) = sse::channel(10);

        tx.send(sse::Data::new("connected")).await.unwrap();
        println!("creating new clients success {:?}", tx);
        self.inner.lock().clients.push(tx);
        rx
    }

    /// Broadcasts `msg` to all clients.
    pub async fn broadcast(&self, msg: &str) {
        let clients = self.inner.lock().clients.clone();

        let send_futures = clients
            .iter()
            .map(|client| client.send(sse::Data::new(msg)));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
    }
}

// async fn index(_req: HttpRequest) -> impl Responder {
//     include_str!("../../../res/index.html")
// }

// Server-Sent Events
pub async fn sse_client(state: web::Data<AppState>) -> impl Responder {
    state.broadcaster.new_client().await
}

// Broadcast message
pub async fn broadcast_msg(
    state: web::Data<AppState>,
    Path((msg,)): Path<(String,)>,
) -> impl Responder {
    state.broadcaster.broadcast(&msg).await;
    HttpResponse::Ok().body("msg sent")
}

// FOR NOW: Disabled so ctrl+c can work.
// #[tokio::main]
// It seems actix_web::main has to be used, or actix_web::rt::spawn panics.
// Will need to work around that.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let manager = SqliteConnectionManager::file("test-drfiley-server.sqlite");
    let pool = r2d2::Pool::new(manager).expect("Couldn't open server DB pool.");
    let conn = pool.get().expect("Could not get DB connection from pool.");
    conn.pragma_update_and_check(Option::None, "journal_mode", "WAL", |_row| Ok(()))
        .expect("Couldn't update journal mode.");
    conn.pragma_update(Option::None, "synchronous", "NORMAL")
        .expect("Couldn't update synchronous mode");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL,
        numbytes INTEGER
    ) STRICT",
        (),
    )
    .expect("Could not create table.");

    let addr = "127.0.0.1:8888"
        .parse()
        .expect("Couldn't parse grpc address");
    let handler = DrFileyHandlerImpl { pool: pool };

    println!("Server listening on {}", addr);

    // TODO: Set up reflection grpc server
    // let reflection_service = tonic_reflection::server::Builder::configure()
    // .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
    // .build()
    // .unwrap();

    let grpc_server = Server::builder()
        .add_service(DrFileyHandlerServer::new(handler))
        // .add_service(reflection_service)
        .serve(addr);

    let broadcaster = Broadcaster::create();

    let h2_server = HttpServer::new(move || {
        let generated = generate();
        App::new()
            .app_data(web::Data::new(AppState {
                broadcaster: Arc::clone(&broadcaster),
            }))
            // This route is used to listen to events/ sse events
            .route("/events{_:/?}", web::get().to(sse_client))
            // This route will create a notification
            .route("/events/{msg}", web::get().to(broadcast_msg))
            .service(ResourceFiles::new("/", generated))

        // App::new().route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    // Note: just using join! here without first using spawn on both futures will cause them
    // to run in one thread. See: https://stackoverflow.com/a/69639766
    tokio::join!(grpc_server, h2_server);
    // h2_server.await?;

    // TODO: Properly pass errors, rather than whatever the above is doing.

    // TODO: Properly handle ctrl+c: https://tokio.rs/tokio/topics/shutdown
    Ok(())
}
