use crate::{Registry, EntityId};

pub trait Query<'a> where Self:Sized {
    fn query(registry:&'a Registry, id:EntityId) -> Option<Self>;
}