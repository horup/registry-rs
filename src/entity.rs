use crate::{Id, World, Component};

pub struct EntityMut<'a> {
    id:Id,
    world:&'a mut World
}

impl<'a> EntityMut<'a> {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn attach<T:Component>(&mut self, component:T) {
        self.world.attach(self.id, component);
    }

    pub fn detach<T:Component>(&mut self) {
        self.world.detach::<T>(self.id);
    }
}