use serde::{Serialize, de::DeserializeOwned};

pub type SingletonId = u8;
pub trait Singleton : Clone + Serialize + DeserializeOwned + 'static + Default {
    fn id() -> SingletonId;
}