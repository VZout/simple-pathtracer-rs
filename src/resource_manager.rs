use std::collections::HashMap;
use std::sync::Arc;

pub struct ResourceManager<R, L>
    where L: ResourceLoader<R>
{
    next_id: u32,
    loader: L,
    cache: HashMap<u32, Arc<R>>,
}

#[allow(dead_code)]
impl<R, L> ResourceManager<R, L>
    where
    L: ResourceLoader<R>,
{
    pub fn new(loader: L) -> Self
    {
        ResourceManager
        {
            next_id: 0,
            loader,
            cache: HashMap::new(),
        }
    }

    pub fn load<D>(&mut self, details: &D) -> u32
        where L: ResourceLoader<R, Args = D>,
              D: ?Sized,
    {
        let id = self.next_id;
        self.next_id += 1;

        match self.loader.load(details)
        {
            Ok(resource) =>
            {
                let resource = Arc::new(resource);
                self.cache.insert(id, resource.clone());
            },
            Err(e) => println!("Failed to load resource: {:?}", e),
        }

        return id;
    }

    pub fn get_mut(&mut self, id: &u32) -> Option<&mut Arc<R>>
    {
        return self.cache.get_mut(&id);
    }

    pub fn set(&mut self, id: u32, item: R)
    {
        let resource = Arc::new(item);
        self.cache.insert(id, resource.clone());
    }

    pub fn get(&self, id: &u32) -> Option<Arc<R>>
    {
        return self.cache.get(&id).cloned();
    }
}

pub trait ResourceLoader<R> {
    type Args: ?Sized;

    fn load(&self, data: &Self::Args) -> Result<R, String>;
}