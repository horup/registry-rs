use crate::{Registry, Component, View};

pub trait RegistryView<'a> {
    fn new(reg:&'a Registry) -> Self;
}

pub struct QIter<'a, RV:RegistryView<'a>> {

}

pub fn query<'a, T:Query<'a>>(&'a self) -> QueryIter<'a, T> {
    QueryIter {
        registry: self,
        keys: self.entities.keys(),
        phantom: PhantomData::default(),
    }
}

impl<'a, T1:Component,> RegistryView<'a> for (View<'a, T1>,) {
    fn new(reg:&'a Registry) -> Self {
        (reg.storage::<T1>(),)
    }
} 

impl<'a, T1:Component, T2:Component> RegistryView<'a> for (View<'a, T1>, View<'a, T2>) {
    fn new(reg:&'a Registry) -> Self {
        (reg.storage::<T1>(), reg.storage::<T2>())
    }
} 

