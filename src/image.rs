use image::{io::Reader as ImageReader, DynamicImage, SubImage};
use image::GenericImageView;
use crate::{Config, Error};

pub struct TileImage {
    pub config: Config,
}

impl TileImage {
    fn create_img(&self) -> Result<DynamicImage, Error> {
        Ok(ImageReader::open(&self.config.filename)?.decode()?)
    }

    pub fn iter(&self) -> TilesIterator {
        let img = &self.create_img().unwrap();
        TilesIterator {
            img: img,
            x_index: 0,
            y_index: 0,
            x_max: img.width() / &self.config.tilesize,
            y_max: img.height() / &self.config.tilesize,
            tilesize: self.config.tilesize,
        }
    }
}

struct TilesIterator<'a> {
    img: &'a DynamicImage,
    x_index: u32,
    y_index: u32,
    x_max: u32,
    y_max: u32,
    tilesize: u32,
}

impl<'a> Iterator for TilesIterator<'a> {
    type Item = SubImage<&'a DynamicImage>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.x_index == self.x_max && self.y_index == self.y_max {
            None
        } else {
            if self.x_index == self.x_max {
                self.x_index = 0;
                self.y_index += 1;
            } else  {
                self.x_index += 1;
            }
            let x1 = self.x_index * self.tilesize;
            let y1 = self.y_index * self.tilesize;
            Some(self.img.view(x1, y1, self.tilesize, self.tilesize))
        }
    }
}
