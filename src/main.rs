use serde::{Serialize, Deserialize};
mod storage;
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

fn main() {
    let mut world = World::new();
    world.register::<Health>();
    world.register::<Position>();

    let e1 = world.spawn();
    world.attach(e1, Health {
        amount:100.0
    });
    world.attach(e1, Position {
        x:1.0,
        y:2.0
    });

    dbg!(world.get::<Health>(e1));


}
