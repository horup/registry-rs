use std::cell::{Ref, RefMut};
use crate::{EntityId, World, Component};

pub struct EntityMut<'a> {
    id:EntityId,
    world:&'a mut World
}

impl<'a> EntityMut<'a> {
    pub fn new(id:EntityId, world:&'a mut World) -> Self {
        Self {
            id,
            world
        }
    }
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn attach<T:Component>(&mut self, component:T) {
        self.world.component_attach(self.id, component);
    }

    pub fn detach<T:Component>(&mut self) {
        self.world.component_detach::<T>(self.id);
    }

    pub fn get<T:Component>(&self) -> Option<Ref<T>> {
        self.world.component::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&self) -> Option<RefMut<T>> {
        self.world.component_mut::<T>(self.id)
    }
}

pub struct Entity<'a> {
    id:EntityId,
    world:&'a World
}

impl<'a> Entity<'a> {
    pub fn new(id:EntityId, world:&'a World) -> Self {
        Self {
            id,
            world
        }
    }
    pub fn id(&self) -> EntityId {
        self.id
    }

    pub fn get<T:Component>(&'a self) -> Option<Ref<'a, T>> {
        self.world.component::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&'a self) -> Option<RefMut<'a, T>> {
        self.world.component_mut::<T>(self.id)
    }
}