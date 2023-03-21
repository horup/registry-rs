use std::collections::HashMap;

use slotmap::SlotMap;

use crate::{Component, Id, Storage};

pub struct World {
    entities:SlotMap<Id, ()>,
    components:HashMap<u16, Storage>
}

impl World {
    pub fn new() -> Self {
        let entities = SlotMap::default();
        let components = HashMap::default();
        Self {
            entities,
            components
        }
    }

    pub fn register<T:Component>(&mut self) {
        if self.components.insert(T::id(), Storage::new::<T>()).is_some() {
            panic!("component type already registered!");
        }
    }

    pub fn attach<T:Component>(&mut self, id:Id, component:T) {
        let storage = self.components.get_mut(&T::id()).expect("component type not registered!");
        unsafe {
            storage.get_mut().insert(id, component);
        }
    }

    pub fn detach<T:Component>(&mut self, id:Id) -> Option<T> {
        let storage = self.components.get_mut(&T::id()).expect("component type not registered!");
        unsafe {
            return storage.get_mut().remove(id);
        }
    }

    pub fn get_mut<T:Component>(&mut self, id:Id) -> Option<&mut T> {
        let storage = self.components.get_mut(&T::id()).expect("component type not registered!");
        unsafe {
            let storage = storage.get_mut();
            return storage.get_mut(id);
        }
    }

    pub fn get<T:Component>(&self, id:Id) -> Option<&T> {
        let storage = self.components.get(&T::id()).expect("component type not registered!");
        unsafe {
            let storage = storage.get();
            return storage.get(id);
        }
    }

    pub fn spawn(&mut self) -> Id {
        self.entities.insert(())
    }

    pub fn despawn(&mut self, id:Id) {
        if self.entities.remove(id).is_some() {

        }
    }
}