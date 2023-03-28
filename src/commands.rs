use crate::Registry;

#[derive(Default)]
pub struct Commands {
    commands:Vec<Box<dyn Fn(&mut Registry) -> ()>>
}

impl Commands {
    pub fn push<T:Fn(&mut Registry) -> () + 'static>(&mut self, f:T) {
        self.commands.push(Box::new(f));
    }

    pub fn execute(&mut self, registry:&mut Registry) {
        for command in self.commands.drain(..) {
            command(registry);
        }
    }
}