use serde::{Serialize, de::DeserializeOwned, Deserialize};
mod storage;
pub use storage::*;
mod component;
pub use component::*;

#[derive(Debug, Serialize, Deserialize)]
struct Health {
    pub amount:f32
}
impl Component for Health {

}

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    pub x:f32,
    pub y:f32
}
impl Component for Position {

}

fn main() {
    let mut storage = Storage::new::<Health>();
    {
        let storage = storage.get_mut();
        for i in 0..5 {
            storage.push(Health {
                amount: i as f32
            });
        }
    }

    let bytes = storage.serialize();
    {
        let storage:&mut Vec<Health> = storage.get_mut();
        storage.clear();
        for e in storage.iter() {
            dbg!(e);
        }
    }

    storage.deserialize(&bytes);

    {
        let storage:&mut Vec<Health> = storage.get_mut();
        for e in storage.iter() {
            dbg!(e);
        }
    }

    
    

    /*let mut v = Vec::new();
    v.push(Health {
        amount:10.0
    });*/
}
