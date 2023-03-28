use std::marker::PhantomData;
use crate::{Registry, EntityId, EntityIter};

pub trait Facade<'a> where Self:Sized {
    fn new(registry:&'a Registry) -> Self;
    fn registry(&self) -> &'a Registry;
    fn query<Q:EntityFacade<'a, Self>>(&'a self) -> EntityFacadeIter<'a, Self, Q> {
        EntityFacadeIter {
            entities:self.registry().iter(),
            facade:self,
            phantom: PhantomData::default()
        }
    }
}

pub trait EntityFacade<'a, T:Facade<'a>> where Self:Sized {
    fn query(facade:&'a T, id:EntityId) -> Option<Self>;
}

pub struct EntityFacadeIter<'a, T:Facade<'a>, Q:EntityFacade<'a, T>> {
    entities:EntityIter<'a>,
    facade:&'a T,
    phantom:PhantomData<Q>
}
impl<'a, T:Facade<'a>, Q:EntityFacade<'a, T>> Iterator for EntityFacadeIter<'a, T, Q> {
    type Item = Q;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        for id in self.entities.by_ref() {
            if let Some(q) = Q::query(self.facade, id) {
                return Some(q);
            }
        }

        None
    }
}