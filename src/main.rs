use std::{time::Instant, cell::RefCell, collections::HashMap};

use serde::{Serialize, Deserialize};
mod storage;
use slotmap::{SecondaryMap, SlotMap};
pub use storage::*;
mod component;
pub use component::*;
mod id;
pub use id::*;
mod world;
pub use world::*;

#[derive(Debug, Serialize, Deserialize)]
struct Health {
    pub amount:f32
}
impl Component for Health {
    fn id() -> u16 {
        1
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    pub x:f32,
    pub y:f32
}
impl Component for Position {
    fn id() -> u16 {
        2
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {

}

impl Component for Player {
    fn id() -> u16 {
        3
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Monster {
}

impl Component for Monster {
    fn id() -> u16 {
        4
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Health>();
    world.register::<Position>();
    world.register::<Player>();
    world.register::<Monster>();

    let size = 1024;
    for i in 0..size {
        let e = world.spawn();
        world.attach(e, Health {
            amount:100.0
        });
        world.attach(e, Position {
            x:i as f32,
            y:i as f32
        });
        world.attach(e, Monster {
        });
    }

    let now = Instant::now();

    for e1 in world.entities() {
        let mut pos1 = world.get_mut::<Position>(e1).unwrap();
        pos1.x += 1.0;
        for e2 in world.entities() {
            if e1 != e2 {
                let mut pos2 = world.get_mut::<Position>(e2).unwrap();
                pos2.y += 1.0;
            }
        }
    }


    let took = Instant::now() - now;
    dbg!(took.as_millis());

    let mut entities:SlotMap<Id,()> = SlotMap::default();
    let mut positions:SecondaryMap<Id, RefCell<Position>> = SecondaryMap::default();  
    let mut hashmap:HashMap<u16, SecondaryMap<Id, RefCell<Position>>> = HashMap::default();
    hashmap.insert(1024, positions);

    for i in 0..size {
        let id = entities.insert(());
        hashmap.get_mut(&1024).unwrap().insert(id, RefCell::new(Position {
            x: 1.0,
            y: 2.0,
        }));
    }

    let now = Instant::now();
    
    for e1 in entities.keys() {
        let pos1 = hashmap.get_mut(&1024).unwrap().get(e1).unwrap();
        pos1.borrow_mut().x += 1.0;
        for e2 in entities.keys() {
            let mut pos2 = hashmap.get_mut(&1024).unwrap().get(e2).unwrap();
            if let Ok(mut pos2) = pos2.try_borrow_mut() {
                pos2.y += 1.0;
            }
        }
    }

    let took = Instant::now() - now;
    dbg!(took.as_millis());


}
