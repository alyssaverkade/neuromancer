mod errors;
mod librarian;
mod services;

use errors::*;
use librarian::Librarian;
use neuromancer::librarian::job_server::*;

pub struct Server {
    router: tonic::transport::server::Router<
        JobServer<Librarian>,
        tonic::transport::server::Unimplemented,
    >,
    addr: String,
}

impl Server {
    const LIBRARIAN_SERVER_ADDRESS: &'static str = "[::1]:1337";
    pub fn new() -> Self {
        let librarian = Librarian::default();
        let router = tonic::transport::Server::builder().add_service(JobServer::new(librarian));
        let addr = Self::LIBRARIAN_SERVER_ADDRESS.to_string();
        Self { addr, router }
    }

    pub async fn build(self) -> Result<()> {
        self.router
            .serve(
                self.addr
                    .parse()
                    .context(InvalidLibrarianAddressSpecified)?,
            )
            .await
            .context(GRPCTransportError)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Server::new().build().await
}
