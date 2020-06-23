use snafu::Snafu;

pub use snafu::{OptionExt, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("{}", source))]
    NeuromancerError {
        source: neuromancer::NeuromancerError,
    },
}

#[derive(Debug, Snafu)]
pub struct SupervisorError(Box<Error>);

impl From<Error> for SupervisorError {
    fn from(err: Error) -> Self {
        Self(Box::new(err))
    }
}

pub type Result<T, E = SupervisorError> = std::result::Result<T, E>;
