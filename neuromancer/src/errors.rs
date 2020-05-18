use snafu::Snafu;

pub use snafu::{OptionExt, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Error decoding protobuf message: {}", source))]
    ProtobufDecodeError { source: prost::DecodeError },
    #[snafu(display("Error encoding protobuf message: {}", source))]
    ProtobufEncodeError { source: prost::EncodeError },
}

#[derive(Debug, Snafu)]
pub struct NeuromancerError(Box<Error>);

impl From<Error> for NeuromancerError {
    fn from(err: Error) -> Self {
        Self(Box::new(err))
    }
}

pub type Result<T, E = NeuromancerError> = std::result::Result<T, E>;
