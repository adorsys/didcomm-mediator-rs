use std::sync::Arc;

use database::Repository;
use didcomm::{did::DIDResolver, FromPrior, Message};
use mongodb::bson::doc;
use serde_json::Error;

use crate::{didcomm::bridge::LocalDIDResolver, model::stateful::entity::Connection};

use super::errors::RotationError;
pub enum Errors {
    Error0(RotationError),
    Error1(Error),
}

pub async fn did_rotation(
    msg: Message,
    conection_repos: &Arc<dyn Repository<Connection>>,
) -> Result<(), Errors> {

    // Check if from_prior is not none
    if msg.from_prior.is_some() {
<<<<<<< Updated upstream
<<<<<<< Updated upstream
=======
        let did_resolver = LocalDIDResolver::def
        let from_prior = FromPrior::unpack(msg.from_prior.as_ref(), did_resolver);
>>>>>>> Stashed changes
=======
        let did_resolver = LocalDIDResolver::def
        let from_prior = FromPrior::unpack(msg.from_prior.as_ref(), did_resolver);
>>>>>>> Stashed changes
        let from_prior: FromPrior =
            serde_json::from_str(&msg.from_prior.unwrap()).map_err(|e| Errors::Error1(e))?;
        let prev = from_prior.iss;

        // validate if did is  known
        let _connection = match conection_repos
            .find_one_by(doc! {"client_did": &prev})
            .await
            .unwrap()
        {
            Some(_connection) => {

                // validate jwt signatures with previous did kid 
            let didresolver = LocalDIDResolver::default();
<<<<<<< Updated upstream
<<<<<<< Updated upstream
=======
            FromPrior::unpack(from_prior_jwt, did_resolver)
>>>>>>> Stashed changes
=======
            FromPrior::unpack(from_prior_jwt, did_resolver)
>>>>>>> Stashed changes
                
            }
            None => {
                return Err(Errors::Error0(RotationError::RotationError))?;
            }
        };
    }
    Ok(())
}
