use std::collections::HashMap;
use std::sync::Arc;

pub struct ResourceManager<R>
    where R: Default + Copy + Clone
{
    next_id: u32,
    cache: HashMap<u32, Arc<R>>,
}

impl<R> ResourceManager<R>
    where R: Default + Copy + Clone
{
    pub fn new() -> Self
    {
        ResourceManager
        {
            next_id: 0,
            cache: HashMap::new(),
        }
    }

    pub fn load(&mut self) -> u32
    {
        let id = self.next_id;
        self.next_id += 1;

        let resource = Arc::new(R::default());
        self.cache.insert(id, resource.clone());

        return id;
    }

    pub fn place(&mut self, r: &R) -> u32
    {
        let id = self.next_id;
        self.next_id += 1;

        let resource = Arc::new(r.clone());
        self.cache.insert(id, resource.clone());

        return id;
    }

    pub fn get(&self, id: &u32) -> Option<Arc<R>>
    {
        return self.cache.get(&id).cloned();
    }
}
