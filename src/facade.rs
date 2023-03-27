use crate::{Registry, EntityId};
pub trait Facade<'a> where Self:Sized {
    fn new(registry:&'a Registry) -> Self;
    fn query(&'a self) {
        let iter = FacadeIter {
            facade:self
        };
    }
}

pub trait FacadeQuery<'a, T:Facade<'a>> where Self:Sized {
    fn query(facade:&T, id:EntityId) -> Option<Self>;
}

pub struct FacadeIter<'a, T:Facade<'a>> {
    pub facade:&'a T
}