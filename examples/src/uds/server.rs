#![cfg_attr(not(unix), allow(unused_imports))]

use std::path::Path;
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    transport::{server::UdsConnectInfo, Server},
    Request, Response, Status,
};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        #[cfg(unix)]
        {
            let conn_info = request.extensions().get::<UdsConnectInfo>().unwrap();
            println!("Got a request {:?} with info {:?}", request, conn_info);

            // Client-side unix sockets are unnamed.
            assert!(conn_info.peer_addr.as_ref().unwrap().is_unnamed());
            // This should contain process credentials for the client socket.
            assert!(conn_info.peer_cred.as_ref().is_some());
        }

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/tmp/tonic/helloworld";

    tokio::fs::create_dir_all(Path::new(path).parent().unwrap()).await?;

    let greeter = MyGreeter::default();

    let uds = UnixListener::bind(path)?;
    let uds_stream = UnixListenerStream::new(uds);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}

#[cfg(not(unix))]
fn main() {
    panic!("The `uds` example only works on unix systems!");
}
