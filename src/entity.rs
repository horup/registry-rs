use std::cell::{Ref, RefMut};

use crate::{Id, World, Component};

pub struct EntityMut<'a> {
    id:Id,
    world:&'a mut World
}

impl<'a> EntityMut<'a> {
    pub fn new(id:Id, world:&'a mut World) -> Self {
        Self {
            id,
            world
        }
    }
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn attach<T:Component>(&mut self, component:T) {
        self.world.attach(self.id, component);
    }

    pub fn detach<T:Component>(&mut self) {
        self.world.detach::<T>(self.id);
    }

    pub fn get<T:Component>(&self) -> Option<Ref<T>> {
        self.world.get::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&mut self) -> Option<RefMut<T>> {
        self.world.get_mut::<T>(self.id)
    }
}

pub struct Entity<'a> {
    id:Id,
    world:&'a World
}

impl<'a> Entity<'a> {
    pub fn new(id:Id, world:&'a World) -> Self {
        Self {
            id,
            world
        }
    }
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn get<T:Component>(&self) -> Option<Ref<T>> {
        self.world.get::<T>(self.id)
    }

    pub fn get_mut<T:Component>(&mut self) -> Option<RefMut<T>> {
        self.world.get_mut::<T>(self.id)
    }
}