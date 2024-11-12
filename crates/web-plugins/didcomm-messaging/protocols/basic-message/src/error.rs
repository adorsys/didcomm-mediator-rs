use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format")]
    InvalidMessageFormat,
    #[error("Encryption or transmission error")]
    TransmissionError,
}
