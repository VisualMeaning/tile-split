use image::{io::Reader as ImageReader, DynamicImage, SubImage};
use image::GenericImageView;
use crate::{Config, Error};

pub struct TileImage<'c> {
    pub config: &'c Config<'c>,
}

impl<'c> TileImage<'c> {
    pub fn create_img(&self) -> Result<DynamicImage, Error> {
        let mut reader = ImageReader::open(&self.config.filename)?;
        // Default memory limit of 512MB is too small for level 6+ PNGs
        reader.no_limits();
        Ok(reader.decode()?)
    }

    pub fn iter<'d>(&self, img: &'d DynamicImage) -> TilesIterator<'d> {
        TilesIterator {
            img,
            x_index: 0,
            y_index: 0,
            x_max: img.width() / &self.config.tilesize,
            y_max: img.height() / &self.config.tilesize,
            tilesize: self.config.tilesize,
        }
    }
}

pub struct TilesIterator<'d> {
    img: &'d DynamicImage,
    x_index: u32,
    y_index: u32,
    x_max: u32,
    y_max: u32,
    tilesize: u32,
}

impl<'d> Iterator for TilesIterator<'d> {
    type Item = (SubImage<&'d DynamicImage>, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        // reaching the end of slicing, return None
        if self.x_index == self.x_max - 1 && self.y_index == self.y_max - 1 {
            None
        } else {
            let x1 = self.x_index * self.tilesize;
            let y1 = self.y_index * self.tilesize;
            // slice image
            let result = (self.img.view(x1, y1, self.tilesize, self.tilesize), self.x_index, self.y_index);
            if self.x_index == self.x_max - 1 {
                // start with a new row
                self.x_index = 0;
                self.y_index += 1;
            } else  {
                // move on to the next block in the row
                self.x_index += 1;
            }
            Some(result)
        }
    }
}
