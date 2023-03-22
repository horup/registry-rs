use serde::{Serialize, de::DeserializeOwned};

pub type ResourceId = u8;
pub trait Resource : Clone + Serialize + DeserializeOwned + 'static {
    fn id() -> ResourceId;
}