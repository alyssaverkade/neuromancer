use std::convert::TryInto;

use tonic::{Request, Response, Status};

use crate::errors::*;
use crate::executor::{Executor, Librarian, ToLibrarian};
use neuromancer::{executor::administrative_server::*, executor::*, write_lock, *};

#[tonic::async_trait]
impl Administrative for Executor {
    async fn librarian_membership_change(
        &self,
        request: Request<LibrarianMembershipChangeRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        // elide bounds checks
        if request.checksum.len() != 8 {
            return Err(Status::out_of_range(
                "length mismatch for checksum".to_string(),
            ));
        }

        let checksum = u64::from_ne_bytes(request.checksum[..].try_into().unwrap());
        match request.librarians.checksum() {
            Ok(computed) if computed != checksum => {
                return Err(Status::invalid_argument(
                    Error::ChecksumMismatch {
                        computed,
                        request: checksum,
                    }
                    .to_string(),
                ))
            }
            Err(e) => return Err(Status::aborted(e.to_string())),
            Ok(_) => (), // all is well
        }

        let mut librarians = write_lock!(self.librarians);

        let new_librarians: Vec<Librarian> = request
            .librarians
            .into_iter()
            .map(|s| s.to_librarian())
            .collect();
        librarians.modify_membership(&new_librarians);
        Ok(Response::new(()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::future::FutureExt;
    use tokio::sync::oneshot;
    use tokio::sync::oneshot::Receiver;

    use super::*;
    use neuromancer::executor::administrative_client::AdministrativeClient;

    const CHECKSUM_MISMATCH_ADDRESS: &str = "[::1]:1337";
    const DEFAULT_STARTUP_TIMEOUT: u64 = 1;
    const LENGTH_MISMATCH_ADDRESS: &str = "[::1]:1336";

    async fn gen_server(addr: &'static str, rx: Receiver<()>) -> tokio::task::JoinHandle<()> {
        let executor = Executor::new();
        let server = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(AdministrativeServer::new(executor))
                .serve_with_shutdown(addr.parse().unwrap(), rx.map(drop))
                .await
                .unwrap();
        });
        tokio::time::delay_for(Duration::from_millis(DEFAULT_STARTUP_TIMEOUT)).await;
        server
    }

    #[tokio::test]
    async fn returns_invalid_argument_on_checksum_mismatch() {
        let (tx, rx) = oneshot::channel::<()>();
        let server = gen_server(CHECKSUM_MISMATCH_ADDRESS, rx).await;
        let mut client_address = String::from("http://");
        client_address += CHECKSUM_MISMATCH_ADDRESS;
        let mut client = AdministrativeClient::connect(client_address).await.unwrap();

        let payload = LibrarianMembershipChangeRequest {
            librarians: vec!["foo".into()],
            checksum: "12345678".as_bytes().into(),
        };

        let err = client
            .librarian_membership_change(Request::new(payload))
            .await
            .unwrap_err();

        assert_eq!(&err.message()[0..29], "checksum mismatch for payload");
        assert_eq!(err.code(), tonic::Code::InvalidArgument);

        tx.send(()).unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn returns_out_of_range_for_checksum_length_mismatch() {
        let (tx, rx) = oneshot::channel::<()>();
        let server = gen_server(LENGTH_MISMATCH_ADDRESS, rx).await;
        let mut client_address = String::from("http://");
        client_address += LENGTH_MISMATCH_ADDRESS;
        let mut client = AdministrativeClient::connect(client_address).await.unwrap();

        let payload = LibrarianMembershipChangeRequest {
            librarians: vec!["foo".into()],
            checksum: "1234".as_bytes().into(),
        };

        let err = client
            .librarian_membership_change(Request::new(payload))
            .await
            .unwrap_err();

        assert_eq!(err.message(), "length mismatch for checksum");
        assert_eq!(err.code(), tonic::Code::OutOfRange);

        tx.send(()).unwrap();
        server.await.unwrap();
    }
}
