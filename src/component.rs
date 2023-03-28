use serde::{Serialize, de::DeserializeOwned};

pub trait Component : Default + Serialize + DeserializeOwned + 'static + Clone {
    fn type_id() -> uuid::Uuid;
}