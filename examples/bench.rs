use std::{time::Instant, cell::RefMut};

use serde::{Serialize, Deserialize};
use registry::{Component, Singleton, SingletonId, Registry, EntityId, Query};

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

struct MonsterQuery<'a> {
    pub position:RefMut<'a, Position>,
    pub monster:RefMut<'a, Monster>
}

impl<'a> Query<'a> for MonsterQuery<'a> {
    fn query(registry:&'a Registry, id:EntityId) -> Option<Self> {
        let position = registry.component_mut::<Position>(id)?;
        let monster = registry.component_mut::<Monster>(id)?;
        Some(Self {
            position,
            monster,
        })
    }
}

fn main() {
    let size = 1000000;
    {
        let mut registry = Registry::new();
        registry.register_component::<Health>();
        registry.register_component::<Position>();
        registry.register_component::<Player>();
        registry.register_component::<Monster>();
        registry.register_singleton::<Global>();
        measure("Registry: creating 1 million monsters", || {
            for i in 0..size {
                let mut e = registry.spawn();
                e.attach(Health {
                    amount:100.0
                });
                e.attach(Position {
                    x:i as f32,
                    y:i as f32
                });
                e.attach(Monster {
                });

                registry.singleton_mut::<Global>().unwrap().monster_count += 1;
            }
        });
        measure("Registry: moving 1 million monsters", || {
            for e in registry.entities() {
                let mut pos = e.get_mut::<Position>().unwrap();
                pos.x += 1.0;
            }
        });
        measure("Registry: moving 1 million monsters using MonsterQuery", || {
            for mut monster in registry.query::<MonsterQuery>() {
                monster.position.x += 1.0;
            }
        });

        let mut bytes = Vec::new();

        measure("Registry: serialize 1 million monsters", || {
            registry.serialize(&mut bytes);
        });

        measure("Registry: clearing all entities, components and resources", || {
            registry.clear();
        });

        measure("Registry: de-serialize 1 million monsters", || {
            registry.deserialize(&bytes);
        });
        
        measure("Registry: clone", || {
            let mut e = registry.spawn();
            e.attach(Player {

            });
            let id = e.id();
            let _e = registry.entity(id).unwrap();
            let _ = registry.clone();
        });
    }
}
