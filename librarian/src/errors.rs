use snafu::Snafu;

pub use snafu::{OptionExt, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("{}", source))]
    NeuromancerError {
        source: neuromancer::NeuromancerError,
    },
    #[snafu(display("invalid address specified for librarian: {}", source))]
    InvalidLibrarianAddressSpecified { source: std::net::AddrParseError },
    #[snafu(display("grpc transport error: {}", source))]
    GRPCTransportError { source: tonic::transport::Error },
    #[snafu(display("uuid encoding error: {}", source))]
    UuidEncodingError { source: uuid::Error },
    #[snafu(display("no identifiers were found for {}", uuid))]
    IdentifierNotFound { uuid: uuid::Uuid },
    #[snafu(display("no identifier provided"))]
    NoIdentifierProvided,
}

#[derive(Debug, Snafu)]
pub struct LibrarianError(Box<Error>);

impl From<Error> for LibrarianError {
    fn from(err: Error) -> Self {
        Self(Box::new(err))
    }
}

pub type Result<T, E = LibrarianError> = std::result::Result<T, E>;
