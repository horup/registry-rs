use std::cell::{RefCell, Ref, RefMut};
use slotmap::SecondaryMap;
use crate::{EntityId, Storage, Component};


pub struct Components<'a, T:Component> {
    storage:&'a SecondaryMap<EntityId, RefCell<T>>
}

impl<'a, T:Component> Components<'a, T> {
    pub unsafe fn new(storage:&'a Storage) -> Self {
        let storage = storage.get();
        Self {
            storage
        }
    }

    pub fn get(&self, id:EntityId) -> Option<Ref<T>> {
        if let Some(c) = self.storage.get(id) {
            if let Ok(c) = c.try_borrow() {
                return Some(c);
            }
        }

        None
    }

    pub fn get_mut(&self, id:EntityId) -> Option<RefMut<T>> {
        if let Some(c) = self.storage.get(id) {
            if let Ok(c) = c.try_borrow_mut() {
                return Some(c);
            }
        }

        None
    }

    pub fn iter(&self) -> Iter<'a, T> {
        let iter = self.storage.iter();
        Iter {
            iter,
        }
    }

    pub fn iter_mut(&self) -> IterMut<'a, T> {
        let iter = self.storage.iter();
        IterMut {
            iter,
        }
    }
}

pub struct Iter<'a, T:Component> {
    iter:slotmap::secondary::Iter<'a, EntityId, RefCell<T>>
}

impl<'a, T:Component> Iterator for Iter<'a, T> {
    type Item = (EntityId, Ref<'a, T>);
    fn next(&mut self) -> Option<Self::Item> {
        for (id, cell) in self.iter.by_ref() {
            if let Ok(value) = cell.try_borrow() {
                return Some((id, value));
            }
        }

        None
    }
}

pub struct IterMut<'a, T:Component> {
    iter:slotmap::secondary::Iter<'a, EntityId, RefCell<T>>
}

impl<'a, T:Component> Iterator for IterMut<'a, T> {
    type Item = (EntityId, RefMut<'a, T>);
    fn next(&mut self) -> Option<Self::Item> {
        for (id, cell) in self.iter.by_ref() {
            if let Ok(value) = cell.try_borrow_mut() {
                return Some((id, value));
            }
        }

        None
    }
}