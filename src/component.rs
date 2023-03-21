use serde::{Serialize, de::DeserializeOwned};

pub trait Component : Serialize + DeserializeOwned + 'static {
    fn id() -> u16;
}