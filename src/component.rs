use serde::{Serialize, de::DeserializeOwned};

pub trait Component : Serialize + DeserializeOwned + 'static + Clone {
    fn id() -> uuid::Uuid;
}