use std::{time::Instant, cell::RefMut};

use serde::{Serialize, Deserialize};
use registry::{Component, Singleton, SingletonId, Registry, EntityId, Query, Facade, Components, FacadeQuery};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Health {
    pub amount:f32
}
impl Component for Health {
    fn id() -> Uuid {
        uuid::uuid!("2cd4dd4a-4585-4d4f-ac58-268125bfdaff")
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Position {
    pub x:f32,
    pub y:f32
}
impl Component for Position {
    fn id() -> Uuid {
        uuid::uuid!("896edd23-0a47-4a84-9eeb-879fe87f8f2e")
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
struct Player {

}

impl Component for Player {
    fn id() -> Uuid {
        uuid::uuid!("09e67821-96be-4dba-89e5-6aef8842ae6d")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Monster {
}

impl Component for Monster {
    fn id() -> Uuid {
        uuid::uuid!("243a0a9b-adb3-4dd4-a0c4-32ee5c3d5164")
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

struct BenchFacade<'a> {
    registry:&'a Registry,
    pub monsters:Components<'a, Monster>,
    pub positions:Components<'a, Position>
}

#[derive(Debug)]
struct MonsterFacade<'a> {
    pub position:RefMut<'a, Position>,
    pub monster:RefMut<'a, Monster>,
}

impl<'a> FacadeQuery<'a, BenchFacade<'a>> for MonsterFacade<'a> {
    fn query(facade:&'a BenchFacade<'a>, id:EntityId) -> Option<Self> {
        let position = facade.positions.get_mut(id)?;
        let monster = facade.monsters.get_mut(id)?;
        Some(Self {
            position,
            monster
        })
    }
}

impl<'a> Facade<'a> for BenchFacade<'a> {
    fn new(registry:&'a Registry) -> Self {
        Self {
            registry,
            monsters:registry.components::<Monster>(),
            positions:registry.components::<Position>()
        }
    }

    fn registry(&self) -> &'a Registry {
        self.registry
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
                let mut pos = registry.component_mut::<Position>(e).unwrap();
                pos.x += 1.0;
            }
        });
        measure("Registry: moving 1 million monsters using MonsterQuery", || {
            for mut monster in registry.query::<MonsterQuery>() {
                monster.position.x += 1.0;
            }
        });

        measure("Registry: moving 1 million monsters using Facade", || {
            let facade = registry.facade::<BenchFacade>();
            for (id, _) in facade.monsters.iter() {
                if let Some(mut pos) = facade.positions.get_mut(id) {
                    pos.x += 1.0;
                }
            }
        });

        measure("Registry: moving 1 million monsters using Facade Query", || {
            let facade = registry.facade::<BenchFacade>();
            for mut monster in facade.query::<MonsterFacade>() {
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
