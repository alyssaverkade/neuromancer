use std::convert::TryInto;

use crossbeam::utils::Backoff;
use neuromancer::{executor::administrative_server::*, executor::*, *};
use tonic::{Request, Response, Status};

use crate::errors::*;
use crate::executor::Executor;

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

        let checksum = u64::from_be_bytes(request.checksum[..].try_into().unwrap());
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

        let backoff = Backoff::new();
        let mut librarians = loop {
            // try to grab the lock
            //
            // because the lock can still be acquired by the other thread between the time
            // we check for the lock being poisoned and the time we actually try to acquire it
            // so that we don't emit futex(2) calls when we know we're contended
            //
            // this was written under the assumption that branch prediction would smooth things
            // out on the fast path while still maintaining correctness
            if !self.librarians.is_poisoned() {
                let librarians = self.librarians.write();
                // lock acquired, yield the value and continue execution
                if librarians.is_ok() {
                    break librarians.unwrap();
                }
            }
            backoff.spin();
        };

        librarians.modify_membership(&request.librarians);
        Ok(Response::new(()))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::time::Duration;

    use futures::future::FutureExt;
    use lazy_static::lazy_static;
    use tokio::sync::oneshot;
    use tokio::sync::oneshot::Receiver;

    use super::*;
    use neuromancer::executor::administrative_client::AdministrativeClient;

    const CHECKSUM_MISMATCH_ADDRESS: &str = "[::1]:1337";

    async fn gen_server(addr: &'static str, rx: Receiver<()>) -> tokio::task::JoinHandle<()> {
        let executor = Executor::default();
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

    const DEFAULT_STARTUP_TIMEOUT: u64 = 1;

    const LENGTH_MISMATCH_ADDRESS: &str = "[::1]:1336";

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
