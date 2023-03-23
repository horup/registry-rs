use std::{ cell::{RefCell, RefMut, Ref}, mem::{MaybeUninit, transmute}, collections::HashMap, io::BufWriter};
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap, basic::Keys};
use crate::{Component, Id, ComponentStorage, ComponentId, EntityMut, Entity, Singleton, SingletonId, SingletonStorage};

const MAX_COMPONENTS:usize = 2_u32.pow(ComponentId::BITS) as usize;
const MAX_SINGLETONS:usize = 2_u32.pow(SingletonId::BITS) as usize;

#[derive(Serialize, Deserialize)]
struct SerializableWorld {
    entities:SlotMap<Id, ()>,
    serialized_components:HashMap<ComponentId, Vec<u8>>,
    serialized_singletons:HashMap<SingletonId, Vec<u8>>
}

pub struct World {
    entities:SlotMap<Id, ()>,
    components:[Option<ComponentStorage>;MAX_COMPONENTS],
    singletons:[Option<SingletonStorage>;MAX_SINGLETONS]
}

pub struct Entities<'a> {
    world:&'a World,
    keys:Keys<'a, Id, ()>
}

impl<'a> Iterator for Entities<'a> {
    type Item = Entity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.keys.next() {
            return Some(Entity::new(id, self.world));
        }

        None
    }
}

impl World {
    pub fn new() -> Self {
        unsafe {
            let entities = SlotMap::default();
            
            let mut data:[MaybeUninit<Option<ComponentStorage>>;MAX_COMPONENTS] = MaybeUninit::uninit().assume_init();
            for elem in &mut data[..] {
                elem.write(None);
            }
            let components = transmute::<_, [Option<ComponentStorage>;MAX_COMPONENTS]>(data);

            let mut data:[MaybeUninit<Option<SingletonStorage>>;MAX_COMPONENTS] = MaybeUninit::uninit().assume_init();
            for elem in &mut data[..] {
                elem.write(None);
            }
            let singletons = transmute::<_, [Option<SingletonStorage>;MAX_COMPONENTS]>(data);
            Self {
                entities,
                components,
                singletons
            }
        }
    }

    pub fn register_singleton<T:Singleton>(&mut self) {
        let i = T::id() as usize;
        if self.singletons[i].is_some() {
            panic!("singleton type already registered!");
        }
        self.singletons[i] = Some(SingletonStorage::new::<T>());
    }

    pub fn singleton<T:Singleton>(&self) -> Option<Ref<T>> {
        unsafe {
            self.singleton_storage::<T>().get::<T>()
        }
    }

    pub fn singleton_mut<T:Singleton>(&self) -> Option<RefMut<T>> {
        unsafe {
            self.singleton_storage::<T>().get_mut::<T>()
        }
    }

    pub fn entities(&self) -> Entities {
        Entities { world: self, keys: self.entities.keys() }
    }

    pub fn entity(&self, id:Id) -> Option<Entity> {
        if self.entities.get(id).is_some() {
            return Some(Entity::new(id, self));
        }
        None
    } 

    pub fn entity_mut(&mut self, id:Id) -> Option<EntityMut> {
        if self.entities.get(id).is_some() {
            return Some(EntityMut::new(id, self));
        }
        None
    }

    pub fn register_component<T:Component>(&mut self) {
        let i = T::id() as usize;
        if self.components[i].is_some() {
            panic!("component type already registered!");
        }
        self.components[i] = Some(ComponentStorage::new::<T>());
    }

    unsafe fn singleton_storage<T:Singleton>(&self) -> &SingletonStorage {
        let i = T::id() as usize;
        return self.singletons.get_unchecked(i).as_ref().expect("singleton type not registered!");
    }

    unsafe fn component_storage_mut<T:Component>(&mut self) -> &mut ComponentStorage {
        let i = T::id() as usize;
        return self.components.get_unchecked_mut(i).as_mut().expect("component type not registered!");
    }

    unsafe fn component_storage<T:Component>(&self) -> &ComponentStorage {
        let i = T::id() as usize;
        return self.components.get_unchecked(i).as_ref().expect("component type not registered!");
    }

    pub fn attach<T:Component>(&mut self, id:Id, component:T) {
        unsafe {
            self.component_storage_mut::<T>().get_mut().insert(id, RefCell::new(component));
        }
    }

    pub fn detach<T:Component>(&mut self, id:Id) -> Option<T> {
        unsafe {
            let cmp:Option<RefCell<T>> = self.component_storage_mut::<T>().get_mut().remove(id);
            if let Some(cmp) = cmp {
                return Some(cmp.into_inner());
            }
            None
        }
    }

    pub fn get_mut<T:Component>(&self, id:Id) -> Option<RefMut<T>> {
        unsafe {
            let storage = self.component_storage::<T>().get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if let Some(cmp) = cmp {
                if let Ok(cmd) = cmp.try_borrow_mut() {
                    return Some(cmd);
                }
            }
            None
        }
    }

    pub fn get<T:Component>(&self, id:Id) -> Option<Ref<T>> {
        unsafe {
            let storage = self.component_storage::<T>().get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if let Some(cmp) = cmp {
                if let Ok(cmd) = cmp.try_borrow() {
                    return Some(cmd);
                }
            }
            None
        }
    }

    pub fn has<T:Component>(&self, id:Id) -> bool {
        unsafe {
            let storage = self.component_storage::<T>().get();
            let cmp:Option<&RefCell<T>> = storage.get(id);
            if cmp.is_some() {
                return true;
            }
            false
        }
    }

    pub fn spawn(&mut self) -> EntityMut {
        let id = self.entities.insert(());
        return EntityMut::new(id, self);
    }

    pub fn despawn(&mut self, id:Id) {
        self.entities.remove(id);
        for storage in self.components.iter_mut() {
            if let Some(storage) = storage {
                storage.remove(id);
            }
        }
    }

    pub fn serialize(&mut self, bytes:&mut Vec<u8>) {
        let mut serialized_components =HashMap::new();
        for index in 0..MAX_COMPONENTS {
            let id = index as ComponentId;
            if let Some(Some(storage)) = self.components.get(index) {
                let mut bytes = Vec::new();
                unsafe {
                    storage.serialize(&mut bytes);
                    serialized_components.insert(id, bytes);
                }
            }
        }
        let mut serialized_singletons =HashMap::new();
        for index in 0..MAX_SINGLETONS {
            let id = index as SingletonId;
            if let Some(Some(storage)) = self.singletons.get(index) {
                let mut bytes = Vec::new();
                unsafe {
                    storage.serialize(&mut bytes);
                    serialized_singletons.insert(id, bytes);
                }
            }
        }
        let w = SerializableWorld {
            entities:self.entities.clone(),
            serialized_components,
            serialized_singletons
        };

        let writer = BufWriter::new(bytes);
        bincode::serialize_into(writer, &w).expect("failed to serialize World");
    }

    pub fn deserialize(&mut self, bytes:&[u8]) {
        let w:SerializableWorld = bincode::deserialize(bytes).expect("failed to deserialize World");
        self.entities = w.entities;
        for (id, bytes) in w.serialized_components.iter() {
            let index = *id as usize;
            if let Some(Some(storage)) = self.components.get_mut(index) {
                unsafe {
                    storage.deserialize(bytes);
                }
            }
        }
        for (id, bytes) in w.serialized_singletons.iter() {
            let index = *id as usize;
            if let Some(Some(storage)) = self.singletons.get_mut(index) {
                unsafe {
                    storage.deserialize(bytes);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        for storage in self.components.iter_mut() {
            if let Some(storage) = storage {
                storage.clear();
            }
        }
        for storage in self.singletons.iter_mut() {
            if let Some(storage) = storage {
                storage.clear();
            }
        }
    }

    pub fn clone(&mut self) -> Self {
        Self { entities: self.entities.clone(), components: self.components.clone(), singletons: self.singletons.clone() }
    }
}