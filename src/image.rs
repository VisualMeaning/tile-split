use image::io::Reader as ImageReader;
use image::GenericImage;
use crate::{Config, Error};


fn save_sub_image(img: &mut image::DynamicImage, x: u32, y: u32, config: &Config)  -> Result<(), Error> {
    let x1 = x * config.tilesize;
    let y1 = y * config.tilesize;
    let sub = img.sub_image(x1, y1, config.tilesize, config.tilesize);
    sub.to_image().save(format!("{p}/{z}_{x}_{y}.png", p=config.folder, z=config.zoomlevel, x = x, y = y))?;
    Ok(())
}

pub fn tile_image(config: Config) -> Result<(), Error> {
    let mut img = ImageReader::open(&config.filename)?.decode()?;

    let x_max = img.width() / config.tilesize;
    let y_max = img.height() / config.tilesize;

    if x_max >= 1 && y_max >=1 {
        for x in 0..x_max {
            for y in 0..y_max {
                save_sub_image(&mut img, x, y, &config)?;
            }
        }
    }
    return Ok(());
}
