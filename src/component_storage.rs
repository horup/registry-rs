use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::io::BufWriter;
use slotmap::SecondaryMap;
use crate::EntityId;
use crate::Component;

pub struct View<'a, T:Component> {
    storage:&'a SecondaryMap<EntityId, RefCell<T>>
}

impl<'a, T:Component> View<'a, T> {
    pub unsafe fn new(storage:&'a EreasedComponentStorage) -> Self {
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

        return None;
    }

    pub fn get_mut(&self, id:EntityId) -> Option<RefMut<T>> {
        if let Some(c) = self.storage.get(id) {
            if let Ok(c) = c.try_borrow_mut() {
                return Some(c);
            }
        }

        return None;
    }
}

pub struct EreasedComponentStorage {
    pub ptr:*mut (),
    pub drop_fn:Box<dyn Fn()>,
    pub serialize_fn:Box<dyn Fn(&mut Vec<u8>)>,
    pub deserialize_fn:Box<dyn Fn(&[u8])>,
    pub remove_fn:Box<dyn Fn(EntityId)>,
    pub clear_fn:Box<dyn Fn()>,
    pub clone_fn:Box<dyn Fn()->Self>
}

impl EreasedComponentStorage {
    pub fn new<T:Component>() -> Self {
        let map:SecondaryMap<EntityId, RefCell<T>> = SecondaryMap::new();
        let boxed = Box::new(map);
        let ptr = Box::into_raw(boxed);
        let f = move || {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        };
        let serialize_fn = move |bytes:&mut Vec<u8>| {
            unsafe {
                let map = ptr.as_ref().unwrap();
                let writer = BufWriter::new(bytes);
                bincode::serialize_into(writer, map).expect("failed to serialize");
            }
        };
        let deserialize_fn = move |bytes:&[u8]| {
            unsafe {
                *ptr.as_mut().unwrap() = bincode::deserialize(bytes).unwrap();
            }
        };
        let remove_fn = move |id:EntityId| {
            unsafe {
                ptr.as_mut().unwrap().remove(id);
            }
        };
        let clear_fn = move || {
            unsafe {
                ptr.as_mut().unwrap().clear();
            }
        };
        let clone_fn = move || {
            let mut new = Self::new::<T>();
            unsafe {
                let org = ptr.as_ref().unwrap();
                let new = new.get_mut::<T>();
                *new = org.clone();
            }

            new
        };
        let ptr = ptr as *mut ();
        Self {
            ptr,
            drop_fn:Box::new(f),
            serialize_fn:Box::new(serialize_fn),
            deserialize_fn:Box::new(deserialize_fn),
            remove_fn:Box::new(remove_fn),
            clear_fn:Box::new(clear_fn),
            clone_fn:Box::new(clone_fn)
        }      
    }
    
    pub unsafe fn get_mut<T>(&mut self) -> &mut SecondaryMap<EntityId, RefCell<T>> {
        let ptr = self.ptr as *mut SecondaryMap<EntityId, RefCell<T>>;
        unsafe {
            return ptr.as_mut().unwrap();
        }
    }

    pub unsafe fn get<T>(&self) -> &SecondaryMap<EntityId, RefCell<T>> {
        let ptr = self.ptr as *const SecondaryMap<EntityId, RefCell<T>>;
        unsafe {
            return ptr.as_ref().unwrap();
        }
    }  

    pub fn remove(&mut self, id:EntityId) {
        self.remove_fn.as_mut()(id);
    }

    pub unsafe fn deserialize(&mut self, bytes:&Vec<u8>) {
        self.deserialize_fn.as_mut()(bytes);
    }

    pub unsafe fn serialize(&self, bytes:&mut Vec<u8>) {
        self.serialize_fn.as_ref()(bytes);
    }

    pub fn clear(&mut self) {
        self.clear_fn.as_mut()();
    }
}

impl Drop for EreasedComponentStorage {
    fn drop(&mut self) {
        self.drop_fn.as_mut()();
    }
}

impl Clone for EreasedComponentStorage {
    fn clone(&self) -> Self {
        self.clone_fn.as_ref()()
    }
}