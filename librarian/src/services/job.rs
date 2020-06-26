use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::errors::*;
use crate::librarian::Librarian;
use neuromancer::{base::*, librarian::job_server::*, librarian::*, read_lock, Checksummable};

#[tonic::async_trait]
impl Job for Librarian {
    async fn identifiers(
        &self,
        request: Request<Identifier>,
    ) -> Result<Response<RunIdentifiers>, Status> {
        let request = request.into_inner();

        if request.uuid.is_empty() {
            return Err(Status::invalid_argument(
                Error::NoIdentifierProvided.to_string(),
            ));
        }

        let uuid = match Uuid::parse_str(&request.uuid) {
            Ok(uuid) => uuid,
            Err(source) => {
                return Err(Status::invalid_argument(
                    Error::UuidEncodingError { source }.to_string(),
                ));
            }
        };

        let graph = read_lock!(self.graph);
        let neighbors = graph.neighbors(uuid);

        // attempt to pre-determine the number of identifiers
        let neighbor_len = {
            let (first, last) = neighbors.size_hint();
            first + last.unwrap_or(0)
        };
        let mut identifiers = Vec::with_capacity(neighbor_len);

        for uuid in neighbors {
            identifiers.push(Identifier {
                uuid: uuid.to_string(),
            });
        }

        let checksum = match identifiers.checksum() {
            Ok(checksum) => checksum.to_ne_bytes().to_vec(),
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };

        let run_identifiers = RunIdentifiers {
            run_ids: identifiers,
            checksum,
        };
        Ok(Response::new(run_identifiers))
    }

    async fn remap(&self, _request: Request<RemapRequest>) -> Result<Response<Identifier>, Status> {
        Ok(Response::new(Identifier {
            ..Default::default()
        }))
    }
}
