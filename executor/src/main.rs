mod errors;
mod executor;
mod services;

use tonic::transport::server::{Router, Unimplemented};

use neuromancer::executor::administrative_server::*;

use crate::errors::*;
use crate::executor::Executor;
use errors::Result;

pub struct Server {
    router: Router<AdministrativeServer<Executor>, Unimplemented>,
    addr: String,
}

impl Server {
    const EXECUTOR_SERVER_ADDRESS: &'static str = "[::1]:9001";

    pub fn new() -> Self {
        let executor = Executor::default();
        let router =
            tonic::transport::Server::builder().add_service(AdministrativeServer::new(executor));
        let addr = Self::EXECUTOR_SERVER_ADDRESS.to_string();
        Self { router, addr }
    }

    pub async fn build(self) -> Result<()> {
        self.router
            .serve(self.addr.parse().context(InvalidAddressForServer)?)
            .await
            .context(GRPCTransport)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Server::new().build().await
}
