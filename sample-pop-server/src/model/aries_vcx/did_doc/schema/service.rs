use crate::model::aries_vcx::did_doc;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use did_doc::error::DidDocumentBuilderError;

use super::{
    types::{uri::Uri, url::Url},
    utils::OneOrList,
};

pub type ServiceTypeAlias = OneOrList<String>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Service<E> {
    id: Uri,
    #[serde(rename = "type")]
    service_type: ServiceTypeAlias,
    service_endpoint: Url,
    #[serde(flatten)]
    extra: E,
}

impl<E> Service<E> {
    pub fn builder(id: Uri, service_endpoint: Url, extra: E) -> ServiceBuilder<E> {
        ServiceBuilder::new(id, service_endpoint, extra)
    }

    pub fn id(&self) -> &Uri {
        &self.id
    }

    pub fn service_type(&self) -> &ServiceTypeAlias {
        &self.service_type
    }

    pub fn service_endpoint(&self) -> &Url {
        &self.service_endpoint
    }

    pub fn extra(&self) -> &E {
        &self.extra
    }
}

#[derive(Debug)]
pub struct ServiceBuilder<E> {
    id: Uri,
    service_endpoint: Url,
    extra: E,
}

#[derive(Debug)]
pub struct ServiceBuilderWithServiceType<E> {
    id: Uri,
    service_type: HashSet<String>,
    service_endpoint: Url,
    extra: E,
}

impl<E> ServiceBuilder<E> {
    pub fn new(id: Uri, service_endpoint: Url, extra: E) -> Self {
        Self {
            id,
            service_endpoint,
            extra,
        }
    }

    pub fn add_service_type(
        self,
        service_type: String,
    ) -> Result<ServiceBuilderWithServiceType<E>, DidDocumentBuilderError> {
        if service_type.is_empty() {
            return Err(DidDocumentBuilderError::MissingField("type"));
        }
        let mut service_types = HashSet::new();
        service_types.insert(service_type);
        Ok(ServiceBuilderWithServiceType {
            id: self.id,
            service_type: service_types,
            service_endpoint: self.service_endpoint,
            extra: self.extra,
        })
    }

    pub fn add_service_types(
        self,
        service_types: Vec<String>,
    ) -> Result<ServiceBuilderWithServiceType<E>, DidDocumentBuilderError> {
        if service_types.is_empty() {
            return Err(DidDocumentBuilderError::MissingField("type"));
        }
        let service_types = service_types.into_iter().collect::<HashSet<_>>();
        Ok(ServiceBuilderWithServiceType {
            id: self.id,
            service_type: service_types,
            service_endpoint: self.service_endpoint,
            extra: self.extra,
        })
    }
}

impl<E> ServiceBuilderWithServiceType<E> {
    pub fn add_service_type(
        mut self,
        service_type: String,
    ) -> Result<Self, DidDocumentBuilderError> {
        if service_type.is_empty() {
            return Err(DidDocumentBuilderError::MissingField("type"));
        }
        self.service_type.insert(service_type);
        Ok(self)
    }

    pub fn build(self) -> Service<E> {
        let service_type = match self.service_type.len() {
            // SAFETY: The only way to get to this state is to add at least one service type
            0 => unreachable!(),
            // SAFETY: We know that the length is non-zero
            1 => OneOrList::One(self.service_type.into_iter().next().unwrap()),
            _ => OneOrList::List(self.service_type.into_iter().collect()),
        };
        Service {
            id: self.id,
            service_type,
            service_endpoint: self.service_endpoint,
            extra: self.extra,
        }
    }
}
