use std::{time::Instant, cell::RefMut};

use serde::{Serialize, Deserialize};
use world::{Component, Singleton, SingletonId, World, EntityId, Query, Entity};

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

impl Component for Monster {
    fn id() -> u8 {
        4
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Global {
    pub monster_count:i32
}

impl Singleton for Global {
    fn id() -> SingletonId {
        1
    }
}


fn measure<F:FnMut()>(name:&str, mut f:F) {
    let now = Instant::now();
    f();
    let elapsed = Instant::now() - now;
    println!("{}ms\t {}", elapsed.as_millis(), name);
}

struct MonsterView<'a> {
    pub position:RefMut<'a, Position>,
    pub monster:RefMut<'a, Monster>
}

impl<'a> Query<'a> for MonsterView<'a> {
    fn from_world(world:&'a World, id:EntityId) -> Option<Self> {
        let position = world.get_mut::<Position>(id)?;
        let monster = world.get_mut::<Monster>(id)?;
        Some(Self {
            position,
            monster,
        })
    }
}
/* 
impl<'a> EntityView for MonsterView<'a> {
    fn from_world(world:&'a World, id:EntityId) -> Option<Self> {
        let mut e = world.entity(id)?;
        let position = e.get_mut::<Position>()?;
        let monster = e.get_mut::<Monster>()?;
        Some(Self {
            id,
            position,
            monster,
        })
    }
}*/

fn main() {
    let size = 1000000;
    {
        let mut world = World::new();
        world.register_component::<Health>();
        world.register_component::<Position>();
        world.register_component::<Player>();
        world.register_component::<Monster>();
        world.register_singleton::<Global>();
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

                world.singleton_mut::<Global>().unwrap().monster_count += 1;
            }
        });
        measure("World: moving 1 million monsters", || {
            for mut e in world.entities() {
                let mut pos = e.get_mut::<Position>().unwrap();
                pos.x += 1.0;
            }
        });
        measure("World: moving 1 million monsters using MonsterView", || {
            for mut monster in world.query::<MonsterView>() {
                monster.position.x += 1.0;
            }
        });

        let mut bytes = Vec::new();

        measure("World: serialize 1 million monsters", || {
            world.serialize(&mut bytes);
        });

        measure("World: clearing all entities, components and resources", || {
            world.clear();
        });

        measure("World: de-serialize 1 million monsters", || {
            world.deserialize(&bytes);
        });
        
        measure("World: clone", || {
            let mut e = world.spawn();
            e.attach(Player {

            });
            let id = e.id();
            let mut e = world.entity(id).unwrap();
            //let mut player = e.get_mut::<Player>().unwrap();
            let _world2 = world.clone();
        });
    }
}
