use axum::{response::{IntoResponse, Response}, http::StatusCode};
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag="type", content="data")]
pub enum Error {
    DecriptionError,
    UnknownTarget,
}


// region: --- impl Error
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RESPONSE");

        // Create a placeholdder for the response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), std::fmt::Error> {
        
        write!(fmt, "{self:?}")
    }
}
// endregion: --- impl Error

// region: --- Client Error
impl Error {
    pub fn client_satus_and_error(&self) -> (StatusCode, ClientError){
        #[allow(unreachable_patterns)]
        match self {
            // -- Auth Errors
            Self::DecriptionError => {
                (StatusCode::UNAUTHORIZED, ClientError::NO_AUTH)
            }

            // -- Model Error
            Self::UnknownTarget => {
                (StatusCode::NOT_FOUND, ClientError::UNKNOWN_TARGET)
            }

            // -- Fallback
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    NO_AUTH,
    UNKNOWN_TARGET,
    SERVICE_ERROR,
}
// endregion: --- Client Error


