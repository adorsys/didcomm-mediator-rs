pub mod stateful;
pub mod traits;

use mongodb::error::Error as MongoError;

use traits::RepositoryError;

impl From<MongoError> for RepositoryError {
    fn from(error: MongoError) -> Self {
        RepositoryError::Generic(error.to_string())
    }
}
