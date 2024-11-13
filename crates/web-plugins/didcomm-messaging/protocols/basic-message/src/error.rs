use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format")]
    InvalidMessageFormat,
    #[error("transmission error")]
    TransmissionError,
}
