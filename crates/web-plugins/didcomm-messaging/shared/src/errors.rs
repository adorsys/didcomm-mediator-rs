use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SharedError {
    #[error("message must be decorated with return route all extension")]
    NoReturnRouteAllDecoration,
}
