use crate::{World, EntityId};

pub trait Query<'a> where Self:Sized {
    fn from_world(world:&'a World, id:EntityId) -> Option<Self>;
}