use serde::{Serialize, de::DeserializeOwned};

pub type ComponentId = uuid::Uuid;

pub trait Component : Serialize + DeserializeOwned + 'static + Clone {
    fn id() -> ComponentId;
}