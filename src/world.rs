use std::{ cell::{RefCell, RefMut, Ref}, mem::{size_of, MaybeUninit, transmute}, path::Components, collections::HashMap, io::BufWriter};
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap, basic::Keys};
use crate::{Component, Id, Storage, ComponentId};

const MAX_COMPONENTS:usize = (2 as u32).pow((size_of::<ComponentId>() * 8) as u32) as usize;

#[derive(Serialize, Deserialize)]
struct SerializableWorld {
    entities:SlotMap<Id, ()>,
    serialized_components:HashMap<ComponentId, Vec<u8>>
}

pub struct World {
    entities:SlotMap<Id, ()>,
    components:[Option<Storage>;MAX_COMPONENTS]//[OptionVec<Option<Storage>>//FxHashMap<u16, Storage>
}

impl World {
    pub fn new() -> Self {
        unsafe {
            let entities = SlotMap::default();
            
            let mut data:[MaybeUninit<Option<Storage>>;MAX_COMPONENTS] = MaybeUninit::uninit().assume_init();
            for elem in &mut data[..] {
                elem.write(None);
            }
            let components = transmute::<_, [Option<Storage>;MAX_COMPONENTS]>(data);
            Self {
                entities,
                components
            }
        }
    }

    pub fn entities(&self) -> Keys<Id, ()> {
        self.entities.keys()
    }

    pub fn register<T:Component>(&mut self) {
        let i = T::id() as usize;
        if self.components[i].is_some() {
            panic!("component type already registered!");
        }
        self.components[i] = Some(Storage::new::<T>());
    }

    unsafe fn storage_mut<T:Component>(&mut self) -> &mut Storage {
        let i = T::id() as usize;
        return self.components.get_unchecked_mut(i).as_mut().expect("component type not registered!");
    }

    unsafe fn storage<T:Component>(&self) -> &Storage {
        let i = T::id() as usize;
        return self.components.get_unchecked(i).as_ref().expect("component type not registered!");
    }

    pub fn attach<T:Component>(&mut self, id:Id, component:T) {
        unsafe {
            self.storage_mut::<T>().get_mut().insert(id, RefCell::new(component));
        }
    }

    pub fn detach<T:Component>(&mut self, id:Id) -> Option<T> {
        unsafe {
            let cmp:Option<RefCell<T>> = self.storage_mut::<T>().get_mut().remove(id);
            if let Some(cmp) = cmp {
                return Some(cmp.into_inner());
            }
            return None;
        }
    }

    pub fn get_mut<T:Component>(&self, id:Id) -> Option<RefMut<T>> {
        unsafe {
            let storage = self.storage::<T>().get();
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
        unsafe {
            let storage = self.storage::<T>().get();
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
        unsafe {
            let storage = self.storage::<T>().get();
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
            for storage in self.components.iter_mut() {
                if let Some(storage) = storage {
                    storage.remove(id);
                }
            }
        }
    }

    pub fn serialize(&self, bytes:&mut Vec<u8>) {
        let mut serialized_components =HashMap::new();
        for index in 0..MAX_COMPONENTS {
            let id = index as ComponentId;
            if let Some(Some(storage)) = self.components.get(index) {
            }
        }
        let w = SerializableWorld {
            entities:self.entities.clone(),
            serialized_components
        };

        let writer = BufWriter::new(bytes);
        bincode::serialize_into(writer, &w).expect("failed to serialize World");
    }

    pub fn deserialize(&mut self, bytes:&[u8]) {
        let w:SerializableWorld = bincode::deserialize(&bytes).expect("failed to deserialize World");
        self.entities = w.entities;
    }
}