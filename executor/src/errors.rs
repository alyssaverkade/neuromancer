use snafu::Snafu;

pub use snafu::{OptionExt, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("{}", source))]
    Neuromancer {
        source: neuromancer::NeuromancerError,
    },
    #[snafu(display("invalid address specified for server: {}", source))]
    InvalidAddressForServer { source: std::net::AddrParseError },
    #[snafu(display("grpc transport error: {}", source))]
    GRPCTransport { source: tonic::transport::Error },
    #[snafu(display(
        "checksum mismatch for payload, computed: {} got: {}",
        computed,
        request
    ))]
    ChecksumMismatch { computed: u64, request: u64 },
}

#[derive(Debug, Snafu)]
pub struct ExecutorError(Box<Error>);

impl From<Error> for ExecutorError {
    fn from(err: Error) -> Self {
        Self(Box::new(err))
    }
}

pub type Result<T, E = ExecutorError> = std::result::Result<T, E>;
