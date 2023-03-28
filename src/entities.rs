use slotmap::basic::Keys;
use slotmap::new_key_type;

new_key_type! {
    pub struct EntityId;
}

pub struct EntityIter<'a> {
    pub(crate) keys:Keys<'a, EntityId, ()>
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next()
    }
}