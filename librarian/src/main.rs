use std::collections::HashMap;

use bytes::BufMut;
use parking_lot::{RwLock, RwLockReadGuard};
use tonic::{Request, Response, Status};
use ulid::Ulid;

mod errors;

use errors::*;
use neuromancer::{base::*, librarian::job_server::*, librarian::*, Checksummable};

type Child = HashMap<Ulid, Vec<Ulid>>;
type Parent = HashMap<Ulid, Option<Ulid>>;
#[derive(Default)]
/// A bit of a hashmap as a service
///
/// FIXME: Currently assumes that there will be a singleton instance running,
/// which is obviously wrong. Need to implement distributed consensus for this
struct Librarian {
    // for parent -> child queries
    child: RwLock<Child>,
    // for child -> parent queries
    // lists the parent of the given child
    parent: RwLock<Parent>,
}

pub struct Server {
    router: tonic::transport::server::Router<
        JobServer<Librarian>,
        tonic::transport::server::Unimplemented,
    >,
    addr: String,
}

#[tonic::async_trait]
impl Job for Librarian {
    async fn identifiers(
        &self,
        request: Request<Identifier>,
    ) -> Result<Response<RunIdentifiers>, Status> {
        let request = request.into_inner();

        let child = self.child.read();

        let ulid = self.validate_ulid(&request.uuid, &child)?;

        let identifiers: Vec<Identifier> = child[&ulid]
            .iter()
            .map(|uuid| Identifier {
                uuid: uuid.to_string(),
            })
            .collect();
        let mut run_identifiers = RunIdentifiers::default();
        run_identifiers.run_ids = identifiers;
        let checksum = match run_identifiers.checksum() {
            Ok(checksum) => checksum,
            Err(e) => return Err(Status::failed_precondition(e.to_string())),
        };
        run_identifiers.checksum.put_u64_le(checksum);
        Ok(Response::new(run_identifiers))
    }

    async fn remap(&self, _request: Request<RemapRequest>) -> Result<Response<Identifier>, Status> {
        Ok(Response::new(Identifier {
            ..Default::default()
        }))
    }
}

impl Librarian {
    fn validate_ulid(&self, ulid: &str, child: &RwLockReadGuard<Child>) -> Result<Ulid, Status> {
        if ulid.is_empty() {
            return Err(Status::invalid_argument(
                Error::NoIdentifierProvided.to_string(),
            ));
        }
        let ulid = match Ulid::from_string(ulid) {
            Ok(ulid) => ulid,
            Err(err) => {
                return Err(Status::invalid_argument(
                    Error::UlidEncodingError { err }.to_string(),
                ))
            }
        };
        // assume hashing twice is less expensive than transforming the list of
        // identifiers to strings
        if !child.contains_key(&ulid) {
            return Err(Status::not_found(
                Error::IdentifierNotFound { ulid }.to_string(),
            ));
        }
        Ok(ulid)
    }

    // fn remap()
}

impl Server {
    const LIBRARIAN_SERVER_ADDRESS: &'static str = "[::1]:443";
    pub fn new() -> Result<Self> {
        let librarian = Librarian::default();
        let router = tonic::transport::Server::builder().add_service(JobServer::new(librarian));
        let addr = Self::LIBRARIAN_SERVER_ADDRESS.to_string();
        Ok(Self { addr, router })
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

fn main() {
    println!("Hello, world!");
}
