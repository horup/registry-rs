use std::{time::Instant, cell::RefMut, process::Command};

use serde::{Serialize, Deserialize};
use registry::{Component, Registry, EntityId, Facade, Components, EntityFacade, Commands};
use uuid::Uuid;

#[derive(Default, Debug, Serialize, Clone, Deserialize)]
struct Health {
    pub amount:f32
}
impl Component for Health {
    fn type_id() -> Uuid {
        uuid::uuid!("2cd4dd4a-4585-4d4f-ac58-268125bfdaff")
    }
}

#[derive(Default, Debug, Serialize, Clone, Deserialize)]
struct Position {
    pub x:f32,
    pub y:f32
}
impl Component for Position {
    fn type_id() -> Uuid {
        uuid::uuid!("896edd23-0a47-4a84-9eeb-879fe87f8f2e")
    }
}

#[derive(Default, Debug, Serialize, Clone, Deserialize)]
struct Player {

}

impl Component for Player {
    fn type_id() -> Uuid {
        uuid::uuid!("09e67821-96be-4dba-89e5-6aef8842ae6d")
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Monster {
}

impl Component for Monster {
    fn type_id() -> Uuid {
        uuid::uuid!("243a0a9b-adb3-4dd4-a0c4-32ee5c3d5164")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Global {
    pub monster_count:i32
}

impl Component for Global {
    fn type_id() -> Uuid {
        uuid::uuid!("243a0a9b-adb3-4dd4-a0c4-32ee5c3d5164")
    }
}


fn measure<F:FnMut()>(name:&str, mut f:F) {
    let now = Instant::now();
    f();
    let elapsed = Instant::now() - now;
    println!("{}ms\t {}", elapsed.as_millis(), name);
}

struct BenchFacade<'a> {
    registry:&'a Registry,
    pub monsters:Components<'a, Monster>,
    pub positions:Components<'a, Position>,
    pub healths:Components<'a, Health>
}

#[derive(Debug)]
struct MonsterFacade<'a> {
    pub position:RefMut<'a, Position>,
    pub _monster:RefMut<'a, Monster>,
    pub _health:RefMut<'a, Health>
}

impl<'a> EntityFacade<'a, BenchFacade<'a>> for MonsterFacade<'a> {
    fn query(facade:&'a BenchFacade<'a>, id:EntityId) -> Option<Self> {
        let position = facade.positions.get_mut(id)?;
        let monster = facade.monsters.get_mut(id)?;
        let health = facade.healths.get_mut(id)?;
        Some(Self {
            position,
            _monster: monster,
            _health: health
        })
    }
}

impl<'a> Facade<'a> for BenchFacade<'a> {
    fn new(registry:&'a Registry) -> Self {
        Self {
            registry,
            monsters:registry.components::<Monster>(),
            positions:registry.components::<Position>(),
            healths:registry.components::<Health>()
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
            let mut hit = 0;
            for e in registry.iter() {
                let mut pos = registry.component_mut::<Position>(e).unwrap();
                pos.x += 1.0;
                hit += 1;
            }

            assert_eq!(hit, size);
        });

        measure("Registry: moving 1 million monsters using Facade", || {
            let facade = registry.facade::<BenchFacade>();
            let mut hit = 0;
            for (id, _) in facade.monsters.iter() {
                if let Some(mut pos) = facade.positions.get_mut(id) {
                    pos.x += 1.0;
                    hit += 1;
                }
            }
        });

        measure("Registry: moving 1 million monsters using Facade Query", || {
            let mut hit = 0;
            let facade = registry.facade::<BenchFacade>();
            for mut monster in facade.query::<MonsterFacade>() {
                monster.position.x += 1.0;  
                hit += 1;
            }
            assert_eq!(hit, size);
        });

        let mut bytes = Vec::new();

        measure("Registry: serialize 1 million monsters", || {
            registry.serialize(&mut bytes);
        });

        measure("Registry: clearing all entities, components and resources", || {
            let before = registry.singleton::<Global>().unwrap().monster_count;
            registry.clear();
            let after = registry.singleton::<Global>().unwrap().monster_count;
            assert_ne!(before, after);
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

        registry.clear();

        measure("Spawning monsters using Commands", || {
            assert_eq!(registry.len(), 0);
            let mut commands = Commands::default();
            for i in 0..size {
                let x = i as f32;
                commands.push(move |reg|{
                    reg.spawn()
                    .attach(Monster {

                    })
                    .attach(Position {
                        x: x,
                        y: 0.0,
                    })
                    .attach(Health {
                        amount:100.0
                    });
                });
            }
            commands.execute(&mut registry);

            assert_eq!(registry.len(), size);
        })
    }
}
