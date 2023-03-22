use std::{time::Instant, cell::RefCell, collections::HashMap};

use serde::{Serialize, Deserialize};
mod component_storage;
use slotmap::{SecondaryMap, SlotMap};
pub use component_storage::*;
mod component;
pub use component::*;
mod id;
pub use id::*;
mod world;
pub use world::*;
mod entity;
pub use entity::*;
mod resource;
pub use resource::*;
pub mod resource_storage;
pub use resource_storage::*;

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Health {
    pub amount:f32
}
impl Component for Health {
    fn id() -> u8 {
        1
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Position {
    pub x:f32,
    pub y:f32
}
impl Component for Position {
    fn id() -> u8 {
        2
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Player {

}

impl Component for Player {
    fn id() -> u8 {
        3
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Monster {
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Global {
}

impl Component for Monster {
    fn id() -> u8 {
        4
    }
}

fn measure<F:FnMut()->()>(name:&str, mut f:F) {
    let now = Instant::now();
    f();
    let elapsed = Instant::now() - now;
    println!("{}", name,);
    println!("{}ms", elapsed.as_millis());
    println!("");
}

fn main() {
    let size = 1000000;
    {
        let mut world = World::new();
        world.register::<Health>();
        world.register::<Position>();
        world.register::<Player>();
        world.register::<Monster>();
        measure("World: creating 1 million monsters", || {
            for i in 0..size {
                let mut e = world.spawn();
                e.attach(Health {
                    amount:100.0
                });
                e.attach(Position {
                    x:i as f32,
                    y:i as f32
                });
                e.attach(Monster {
                });
            }
        });
        measure("World: moving 1 million monsters", || {
            for mut e in world.entities() {
                let mut pos = e.get_mut::<Position>().unwrap();
                pos.x += 1.0;
            }
        });

        let mut bytes = Vec::new();

        measure("World: serialize 1 million monsters", || {
            world.serialize(&mut bytes);
        });

        measure("World: clearing all entnties and components", || {
            world.clear();
        });

        measure("World: de-serialize 1 million monsters", || {
            world.deserialize(&bytes);
        });
        
        measure("World: clone", || {
            let mut world2 = world.clone();
        });
    }
/*
    {
        let mut entities:SlotMap<Id,()> = SlotMap::default();
        let mut positions:SecondaryMap<Id, Position> = SecondaryMap::default();  

        for i in 0..size {
            let id = entities.insert(());
            positions.insert(id, Position {
                x: i as f32,
                y: i as f32,
            });
        }

        measure("Slotmap: moving 1 million monsters", || {
            for e in entities.keys() {
                let mut pos = positions.get_mut(e).unwrap();
                pos.x += 1.0;
            }
        });
    }

    {
        let mut entities:SlotMap<Id,()> = SlotMap::default();
        let mut positions:SecondaryMap<Id, RefCell<Position>> = SecondaryMap::default();  

        for i in 0..size {
            let id = entities.insert(());
            positions.insert(id, RefCell::new(Position {
                x: i as f32,
                y: i as f32,
            }));
        }

        measure("Slotmap Refcell: moving 1 million monsters", || {
            for e in entities.keys() {
                let mut pos = positions.get(e).unwrap().borrow_mut();
                pos.x += 1.0;
            }
        });

        measure("Slotmap Refcell: moving 1 million monsters using positions iter", || {
            for (id, mut pos) in positions.iter_mut() {
                pos.borrow_mut().x += 1.0;
            }
        });
    }

    {
        let mut vec = Vec::new();
        for i in 0..size {
            vec.push(RefCell::new(Position {
                x: i as f32,
                y: i as f32,
            }));
        }

        measure("Vec moving 1 million monsters", || {
            for pos in vec.iter_mut() {
                pos.borrow_mut().x += 1.0;
            }
        });
    }
    */
}
