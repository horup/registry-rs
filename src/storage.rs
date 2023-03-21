use crate::Component;

pub struct Storage {
    pub ptr:*mut (),
    pub drop_fn:Box<dyn Fn() -> ()>,
    pub serialize_fn:Box<dyn Fn()->Vec<u8>>,
    pub deserialize_fn:Box<dyn Fn(&Vec<u8>)>
}

impl Storage {
    pub fn new<T:Component>() -> Self {
        let vec:Vec<T> = Vec::new();
        let boxed = Box::new(vec);
        let ptr = Box::into_raw(boxed);
        let f = move || {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        };
        let serialize_fn = move || {
            unsafe {
                let vec = ptr.as_ref().unwrap();
                let bytes = bincode::serialize(vec).unwrap();
                return bytes;
            }
        };
        let deserialize_fn = move |bytes:&Vec<u8>| {
            unsafe {
                *ptr.as_mut().unwrap() = bincode::deserialize(&bytes).unwrap();
            }
        };
        let ptr = ptr as *mut ();
        Self {
            ptr,
            drop_fn:Box::new(f),
            serialize_fn:Box::new(serialize_fn),
            deserialize_fn:Box::new(deserialize_fn)
        }      
    }

    pub fn get_mut<T>(&mut self) -> &mut Vec<T> {
        let ptr = self.ptr as *mut Vec<T>;
        unsafe {
            return ptr.as_mut().unwrap();
        }
    } 

    pub fn deserialize(&mut self, bytes:&Vec<u8>) {
        self.deserialize_fn.as_mut()(bytes);
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.serialize_fn.as_ref()()
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        self.drop_fn.as_mut()();
    }
}