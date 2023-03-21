use std::{collections::HashMap, cell::{RefCell, RefMut, Ref}};
use slotmap::{SlotMap, basic::Keys};
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

    pub fn entities(&self) -> Keys<Id, ()> {
        self.entities.keys()
    }

    pub fn register<T:Component>(&mut self) {
        if self.components.insert(T::id(), Storage::new::<T>()).is_some() {
            panic!("component type already registered!");
        }
    }

    pub fn attach<T:Component>(&mut self, id:Id, component:T) {
        let storage = self.components.get_mut(&T::id()).expect("component type not registered!");
        unsafe {
            storage.get_mut().insert(id, RefCell::new(component));
        }
    }

    pub fn detach<T:Component>(&mut self, id:Id) -> Option<T> {
        let storage = self.components.get_mut(&T::id()).expect("component type not registered!");
        unsafe {
            let cmp:Option<RefCell<T>> = storage.get_mut().remove(id);
            if let Some(cmp) = cmp {
                return Some(cmp.into_inner());
            }
            return None;
        }
    }

    pub fn get_mut<T:Component>(&self, id:Id) -> Option<RefMut<T>> {
        let storage = self.components.get(&T::id()).expect("component type not registered!");
        unsafe {
            let storage = storage.get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if let Some(cmp) = cmp {
                if let Ok(cmd) = cmp.try_borrow_mut() {
                    return Some(cmd);
                }
            }
            return None;
        }
    }

    pub fn get<T:Component>(&self, id:Id) -> Option<Ref<T>> {
        let storage = self.components.get(&T::id()).expect("component type not registered!");
        unsafe {
            let storage = storage.get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if let Some(cmp) = cmp {
                if let Ok(cmd) = cmp.try_borrow() {
                    return Some(cmd);
                }
            }
            return None;
        }
    }

    pub fn has<T:Component>(&self, id:Id) -> bool {
        let storage = self.components.get(&T::id()).expect("component type not registered!");
        unsafe {
            let storage = storage.get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if cmp.is_some() {
                return true;
            }
            return false;
        }
    }

    pub fn spawn(&mut self) -> Id {
        self.entities.insert(())
    }

    pub fn despawn(&mut self, id:Id) {
        if self.entities.remove(id).is_some() {
            for (_, storage) in self.components.iter_mut() {
                storage.remove(id);
            }
        }
    }
}