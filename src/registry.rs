use std::{ cell::{RefCell, RefMut, Ref}, mem::{MaybeUninit, transmute}, collections::HashMap, io::BufWriter, marker::PhantomData, any::type_name};
use fxhash::FxHashMap;
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap, basic::Keys};
use uuid::Uuid;
use crate::{Component, EntityId, ComponentStorage, ComponentId, EntityMut, Entity, Singleton, SingletonId, SingletonStorage, Query};

const MAX_SINGLETONS:usize = 2_u32.pow(SingletonId::BITS) as usize;

#[derive(Serialize, Deserialize)]
struct SerializableRegistry {
    entities:SlotMap<EntityId, ()>,
    serialized_components:HashMap<ComponentId, Vec<u8>>,
    serialized_singletons:HashMap<SingletonId, Vec<u8>>
}

pub struct Registry {
    entities:SlotMap<EntityId, ()>,
    components:FxHashMap<Uuid, ComponentStorage>,
    singletons:[Option<SingletonStorage>;MAX_SINGLETONS]
}

pub struct Entities<'a> {
    registry:&'a Registry,
    keys:Keys<'a, EntityId, ()>
}

impl<'a> Iterator for Entities<'a> {
    type Item = Entity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.keys.next() {
            return Some(Entity::new(id, self.registry));
        }

        None
    }
}

pub struct QueryIter<'a, T> where T:Sized + Query<'a> {
    registry:&'a Registry,
    keys:Keys<'a, EntityId, ()>,
    phantom:PhantomData<T>
}

impl<'a, T> Iterator for QueryIter<'a, T> where T:Sized + Query<'a> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.keys.next() {
            if let Some(q) = T::query(self.registry, id) {
                return Some(q);
            }
        }

        None
    }
}

impl Registry {
    pub fn new() -> Self {
        unsafe {
            let entities = SlotMap::default();
            let components = FxHashMap::default();

            let mut data:[MaybeUninit<Option<SingletonStorage>>;MAX_SINGLETONS] = MaybeUninit::uninit().assume_init();
            for elem in &mut data[..] {
                elem.write(None);
            }
            let singletons = transmute::<_, [Option<SingletonStorage>;MAX_SINGLETONS]>(data);
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
            panic!("{} singleton already registered!", type_name::<T>());
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
        Entities { registry: self, keys: self.entities.keys() }
    }

    pub fn query<'a, T:Query<'a>>(&'a self) -> QueryIter<'a, T> {
        QueryIter {
            registry: self,
            keys: self.entities.keys(),
            phantom: PhantomData::default(),
        }
    }

    pub fn entity(&self, id:EntityId) -> Option<Entity> {
        if self.entities.get(id).is_some() {
            return Some(Entity::new(id, self));
        }
        None
    } 

    pub fn entity_mut(&mut self, id:EntityId) -> Option<EntityMut> {
        if self.entities.get(id).is_some() {
            return Some(EntityMut::new(id, self));
        }
        None
    }

    pub fn register_component<T:Component>(&mut self) {
        let id = T::id();
        if self.components.get(&id).is_some() {
            panic!("{} component already registered!", type_name::<T>());
        }
        self.components.insert(id, ComponentStorage::new::<T>());
    }

    unsafe fn singleton_storage<T:Singleton>(&self) -> &SingletonStorage {
        let i = T::id() as usize;
        match self.singletons.get_unchecked(i).as_ref() {
            Some(storage) => storage,
            None => panic!("{} singleton type not registered!", type_name::<T>()),
        }
    }

    unsafe fn component_storage_mut<T:Component>(&mut self) -> &mut ComponentStorage {
        let id = T::id();
        match self.components.get_mut(&id) {
            Some(storage) => storage,
            None => panic!("{} component type not registered!", type_name::<T>()),
        }
    }

    unsafe fn component_storage<T:Component>(&self) -> &ComponentStorage {
        let id = T::id();
        match self.components.get(&id) {
            Some(storage) => storage,
            None => panic!("{} component type not registered!", type_name::<T>()),
        }
    }

    pub fn component_attach<T:Component>(&mut self, id:EntityId, component:T) {
        unsafe {
            self.component_storage_mut::<T>().get_mut().insert(id, RefCell::new(component));
        }
    }

    pub fn component_detach<T:Component>(&mut self, id:EntityId) -> Option<T> {
        unsafe {
            let cmp:Option<RefCell<T>> = self.component_storage_mut::<T>().get_mut().remove(id);
            if let Some(cmp) = cmp {
                return Some(cmp.into_inner());
            }
            None
        }
    }

    pub fn component_mut<T:Component>(&self, id:EntityId) -> Option<RefMut<T>> {
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

    pub fn component<T:Component>(&self, id:EntityId) -> Option<Ref<T>> {
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

    pub fn component_has<T:Component>(&self, id:EntityId) -> bool {
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

    pub fn despawn(&mut self, id:EntityId) {
        self.entities.remove(id);
        for (_, storage) in self.components.iter_mut() {
            if let storage = storage {
                storage.remove(id);
            }
        }
    }

    pub fn serialize(&mut self, bytes:&mut Vec<u8>) {
        let mut serialized_components =HashMap::new();
        for (id, storage) in self.components.iter() {
            let mut bytes = Vec::new();
            unsafe {
                storage.serialize(&mut bytes);
                serialized_components.insert(*id, bytes);
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
        let w = SerializableRegistry {
            entities:self.entities.clone(),
            serialized_components,
            serialized_singletons
        };

        let writer = BufWriter::new(bytes);
        bincode::serialize_into(writer, &w).expect("failed to serialize Registry");
    }

    pub fn deserialize(&mut self, bytes:&[u8]) {
        let w:SerializableRegistry = bincode::deserialize(bytes).expect("failed to deserialize Registry");
        self.entities = w.entities;
        for (id, bytes) in w.serialized_components.iter() {
            if let Some(storage) = self.components.get_mut(id) {
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
        for (_, storage) in self.components.iter_mut() {
            storage.clear();
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