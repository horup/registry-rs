use std::{ cell::{RefCell, RefMut, Ref}, collections::HashMap, io::BufWriter, any::type_name, mem::replace};
use fxhash::FxHashMap;
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap};
use uuid::Uuid;
use crate::{Component, EntityId, Storage, EntityMut, Entity, Components, Facade, EntityIter, Commands};

#[derive(Serialize, Deserialize)]
struct SerializableRegistry {
    entities:SlotMap<EntityId, ()>,
    serialized_components:HashMap<Uuid, Vec<u8>>,
    serialized_singletons:HashMap<Uuid, Vec<u8>>
}

pub struct Registry {
    commands:RefCell<Commands>,
    entities:SlotMap<EntityId, ()>,
    singleton:EntityId,
    components:FxHashMap<Uuid, Storage>,
    singletons:FxHashMap<Uuid, Storage>,
}

impl Registry {
    pub fn new() -> Self {
        let entities = SlotMap::default();
        let components = FxHashMap::default();
        let singletons = FxHashMap::default();
        let singleton = SlotMap::<EntityId, ()>::default().insert(());
        Self {
            entities,
            components,
            singletons,
            singleton,
            commands:RefCell::new(Commands::default())
        }
    }

    pub fn push<F:Fn(&mut Self)->() + 'static>(&self, f:F) {
        self.commands.borrow_mut().push(Box::new(f));
    }

    pub fn execute(&mut self) {
        let commands = replace(&mut self.commands, RefCell::new(Commands::default()));
        commands.borrow_mut().execute(self);
    }

    pub fn facade<'a, T:Facade<'a>>(&'a self) -> T {
        T::new(self)
    }

    pub fn register_singleton<T:Component + Default>(&mut self) {
        let id = T::type_id();
        if self.singletons.get(&id).is_some() {
            panic!("{} singleton already registered!", type_name::<T>());
        }
        unsafe {
            let mut storage = Storage::new::<T>();
            let view = storage.get_mut::<T>();
            view.insert(self.singleton, RefCell::new(T::default()));
            self.singletons.insert(id, storage);
        }
    }

    pub fn singleton<T:Component>(&self) -> Option<Ref<T>> {
        unsafe {
            if let Some(cell) = self.singleton_storage::<T>().get::<T>().get(self.singleton) {
                if let Ok(cell) = cell.try_borrow() {
                    return Some(cell);
                }
            }

            None
        }
    }

    pub fn singleton_mut<T:Component>(&self) -> Option<RefMut<T>> {
        unsafe {
            if let Some(cell) = self.singleton_storage::<T>().get::<T>().get(self.singleton) {
                if let Ok(cell) = cell.try_borrow_mut() {
                    return Some(cell);
                }
            }

            None
        }
    }

    pub fn iter(&self) -> EntityIter {
        EntityIter { keys: self.entities.keys() }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
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
        let id = T::type_id();
        if self.components.get(&id).is_some() {
            panic!("{} component already registered!", type_name::<T>());
        }
        self.components.insert(id, Storage::new::<T>());
    }

    unsafe fn singleton_storage<T:Component>(&self) -> &Storage {
        let id = T::type_id();
        match self.singletons.get(&id) {
            Some(storage) => storage,
            None => panic!("{} singleton type not registered!", type_name::<T>()),
        }
    }

    unsafe fn component_storage_mut<T:Component>(&mut self) -> &mut Storage {
        let id = T::type_id();
        match self.components.get_mut(&id) {
            Some(storage) => storage,
            None => panic!("{} component type not registered!", type_name::<T>()),
        }
    }

    unsafe fn component_storage<T:Component>(&self) -> &Storage {
        let id = T::type_id();
        match self.components.get(&id) {
            Some(storage) => storage,
            None => panic!("{} component type not registered!", type_name::<T>()),
        }
    }

    pub fn components<T:Component>(&self) -> Components<T> {
        let id = T::type_id();
        match self.components.get(&id) {
            Some(storage) => unsafe { Components::new(storage) },
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
            storage.remove(id);
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
        for (id, storage) in self.singletons.iter() {
            let mut bytes = Vec::new();
            unsafe {
                storage.serialize(&mut bytes);
                serialized_singletons.insert(*id, bytes);
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
            if let Some(storage) = self.singletons.get_mut(id) {
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
        for (_, storage) in self.singletons.iter_mut() {
            storage.default(self.singleton);
        }
    }

    pub fn clone(&mut self) -> Self {
        Self { entities: self.entities.clone(), components: self.components.clone(), singletons: self.singletons.clone(), singleton:self.singleton, commands:RefCell::new(Commands::default()) }
    }
}