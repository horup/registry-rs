use crate::{World, EntityId};

pub trait Query<'a> where Self:Sized {
    fn query(world:&'a World, id:EntityId) -> Option<Self>;
}