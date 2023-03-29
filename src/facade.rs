use crate::{Registry, EntityId, EntityIter};

pub trait Facade<'a> where Self:Sized {
    fn new(registry:&'a Registry) -> Self;
    fn registry(&self) -> &'a Registry;
    fn query<EF:EntityFacade<'a, Facade = Self>>(&'a self) -> EntityFacadeIter<'a, EF> {
        EntityFacadeIter {
            entities:self.registry().iter(),
            facade:self,
        }
    }
}

pub trait EntityFacade<'a> where Self:Sized  {
    type Facade : Facade<'a>;
    fn query(facade:&'a Self::Facade, id:EntityId) -> Option<Self>;
}

pub struct EntityFacadeIter<'a, EF:EntityFacade<'a>> {
    entities:EntityIter<'a>,
    facade:&'a EF::Facade
}
impl<'a, EF:EntityFacade<'a>> Iterator for EntityFacadeIter<'a, EF> {
    type Item = EF;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        for id in self.entities.by_ref() {
            if let Some(q) = EF::query(&self.facade, id) {
                return Some(q);
            }
        }

        None
    }
}