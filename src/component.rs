use serde::{Serialize, de::DeserializeOwned};

pub type ComponentId = u8;

pub trait Component : Serialize + DeserializeOwned + 'static + Clone {
    fn id() -> ComponentId;
}