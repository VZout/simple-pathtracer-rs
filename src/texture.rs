extern crate image;

use crate::resource_manager::*;

pub type Texture = image::DynamicImage;
pub type TextureManager = ResourceManager<Texture, TextureLoader>;

pub struct TextureLoader
{
}

impl ResourceLoader<Texture> for TextureLoader {
    type Args = str;
    fn load(&self, path: &str) -> Result<Texture, String> {
        let img = image::open(path).unwrap();
        return Ok(img);
    }
}