use slotmap::SecondaryMap;
use crate::Id;
use crate::Component;

pub struct Storage {
    pub ptr:*mut (),
    pub drop_fn:Box<dyn Fn() -> ()>,
    pub serialize_fn:Box<dyn Fn()->Vec<u8>>,
    pub deserialize_fn:Box<dyn Fn(&Vec<u8>)>,
    pub remove_fn:Box<dyn Fn(Id)>
}

impl Storage {
    pub fn new<T:Component>() -> Self {
        let map:SecondaryMap<Id, T> = SecondaryMap::new();
        let boxed = Box::new(map);
        let ptr = Box::into_raw(boxed);
        let f = move || {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        };
        let serialize_fn = move || {
            unsafe {
                let map = ptr.as_ref().unwrap();
                let bytes = bincode::serialize(map).unwrap();
                return bytes;
            }
        };
        let deserialize_fn = move |bytes:&Vec<u8>| {
            unsafe {
                *ptr.as_mut().unwrap() = bincode::deserialize(&bytes).unwrap();
            }
        };
        let remove_fn = move |id:Id| {
            unsafe {
                ptr.as_mut().unwrap().remove(id);
            }
        };
        let ptr = ptr as *mut ();
        Self {
            ptr,
            drop_fn:Box::new(f),
            serialize_fn:Box::new(serialize_fn),
            deserialize_fn:Box::new(deserialize_fn),
            remove_fn:Box::new(remove_fn)
        }      
    }

    pub unsafe fn get_mut<T>(&mut self) -> &mut SecondaryMap<Id, T> {
        let ptr = self.ptr as *mut SecondaryMap<Id, T>;
        unsafe {
            return ptr.as_mut().unwrap();
        }
    }

    pub unsafe fn get<T>(&self) -> &SecondaryMap<Id, T> {
        let ptr = self.ptr as *const SecondaryMap<Id, T>;
        unsafe {
            return ptr.as_ref().unwrap();
        }
    }  

    pub unsafe fn deserialize(&mut self, bytes:&Vec<u8>) {
        self.deserialize_fn.as_mut()(bytes);
    }

    pub unsafe fn serialize(&self) -> Vec<u8> {
        self.serialize_fn.as_ref()()
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        self.drop_fn.as_mut()();
    }
}