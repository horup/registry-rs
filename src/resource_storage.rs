use std::borrow::BorrowMut;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::io::BufWriter;
use crate::Resource;

pub struct ResourceStorage {
    pub ptr:*mut (),
    pub drop_fn:Box<dyn Fn() -> ()>,
    pub serialize_fn:Box<dyn Fn(&mut Vec<u8>)>,
    pub deserialize_fn:Box<dyn Fn(&[u8])>,
    pub clone_fn:Box<dyn Fn()->Self>,
    pub clear_fn:Box<dyn Fn()>,
}

impl ResourceStorage {
    pub fn new<T:Resource>() -> Self {
        let resource = RefCell::new(T::default());
        let boxed = Box::new(resource);
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
        let clone_fn = move || {
            unsafe {
                let org = ptr.as_ref().unwrap();
                let mut new = Self::new::<T>();
                *new.get_mut().unwrap() = org.clone();
                return new;
            }
        };
        let clear_fn = move || {
            unsafe {
                let ref_cell = ptr.as_mut().unwrap().borrow_mut();
                ref_cell.replace(T::default());
            }
        };
        let ptr = ptr as *mut ();
        Self {
            ptr,
            drop_fn:Box::new(f),
            serialize_fn:Box::new(serialize_fn),
            deserialize_fn:Box::new(deserialize_fn),
            clone_fn:Box::new(clone_fn),
            clear_fn:Box::new(clear_fn)
        }      
    }
    
    pub unsafe fn get<T>(&mut self) -> Option<Ref<T>> {
        let ptr = self.ptr as *mut RefCell<T>;
        unsafe {
            let cell = ptr.as_ref().unwrap().try_borrow();
            if let Ok(cell) = cell {
                return Some(cell);
            }

            return None;
        }
    }

    pub unsafe fn get_mut<T>(&self) -> Option<RefMut<T>> {
        let ptr = self.ptr as *mut RefCell<T>;
        unsafe {
            let cell = ptr.as_mut().unwrap().try_borrow_mut();
            if let Ok(cell) = cell {
                return Some(cell);
            }

            return None;
        }
    }  

    pub fn clear(&mut self) {
        self.clear_fn.as_mut()();
    }

    pub unsafe fn deserialize(&mut self, bytes:&Vec<u8>) {
        self.deserialize_fn.as_mut()(bytes);
    }

    pub unsafe fn serialize(&self, bytes:&mut Vec<u8>) {
        self.serialize_fn.as_ref()(bytes);
    }
}

impl Drop for ResourceStorage {
    fn drop(&mut self) {
        self.drop_fn.as_mut()();
    }
}

impl Clone for ResourceStorage {
    fn clone(&self) -> Self {
        self.clone_fn.as_ref()()
    }
}