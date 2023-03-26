use std::cell::{Ref, RefMut};
use crate::{EntityId, Registry, Component};

pub struct EntityMut<'a> {
    id:EntityId,
    registry:&'a mut Registry
}

impl<'a> EntityMut<'a> {
    pub fn new(id:EntityId, registry:&'a mut Registry) -> Self {
        Self {
            id,
            registry
        }
    }
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn attach<T:Component>(&mut self, component:T) {
        self.registry.component_attach(self.id, component);
    }

    pub fn detach<T:Component>(&mut self) {
        self.registry.component_detach::<T>(self.id);
    }

    pub fn get<T:Component>(&self) -> Option<Ref<T>> {
        self.registry.component::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&self) -> Option<RefMut<T>> {
        self.registry.component_mut::<T>(self.id)
    }
}

pub struct Entity<'a> {
    id:EntityId,
    registry:&'a Registry
}

impl<'a> Entity<'a> {
    pub fn new(id:EntityId, registry:&'a Registry) -> Self {
        Self {
            id,
            registry
        }
    }
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn get<T:Component>(&'a self) -> Option<Ref<'a, T>> {
        self.registry.component::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&'a self) -> Option<RefMut<'a, T>> {
        self.registry.component_mut::<T>(self.id)
    }
}