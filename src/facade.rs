use std::marker::PhantomData;
use crate::{Registry, EntityId, Entities};

pub trait Facade<'a> where Self:Sized {
    fn new(registry:&'a Registry) -> Self;
    fn registry(&self) -> &'a Registry;
    fn query<Q:FacadeQuery<'a, Self>>(&'a self) -> FacadeIter<'a, Self, Q> {
        FacadeIter {
            entities:self.registry().entities(),
            facade:self,
            phantom: PhantomData::default()
        }
    }
}

pub trait FacadeQuery<'a, T:Facade<'a>> where Self:Sized {
    fn query(facade:&T, id:EntityId) -> Option<Self>;
}

pub struct FacadeIter<'a, T:Facade<'a>, Q:FacadeQuery<'a, T>> {
    entities:Entities<'a>,
    facade:&'a T,
    phantom:PhantomData<Q>
}
impl<'a, T:Facade<'a>, Q:FacadeQuery<'a, T>> Iterator for FacadeIter<'a, T, Q> {
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.entities.next() {
            if let Some(q) = Q::query(self.facade, id.id()) {
                return Some(q);
            }
        }

        None
    }
}