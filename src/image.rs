use crate::{Config, Error};
use image::GenericImageView;
use image::{io::Reader as ImageReader, DynamicImage, SubImage};
use zorder::coord_of;

pub struct TileImage<'c> {
    pub config: &'c Config<'c>,
}

impl<'c> TileImage<'c> {
    pub fn open_img(&self) -> Result<DynamicImage, Error> {
        let mut reader = ImageReader::open(self.config.filename)?;
        // Default memory limit of 512MB is too small for level 6+ PNGs
        reader.no_limits();
        Ok(reader.decode()?)
    }

    pub fn iter<'d>(&self, img: &'d DynamicImage) -> TilesIterator<'d> {
        TilesIterator {
            img,
            morton_idx: 0,
            morton_idx_max: img.width() / self.config.tilesize * img.height()
                / self.config.tilesize,
            tilesize: self.config.tilesize,
        }
    }
}

pub struct TilesIterator<'d> {
    img: &'d DynamicImage,
    morton_idx: u32,
    morton_idx_max: u32,
    tilesize: u32,
}

impl<'d> Iterator for TilesIterator<'d> {
    type Item = (SubImage<&'d DynamicImage>, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        // reaching the end of slicing, return None
        let coord = coord_of(self.morton_idx);
        let x = coord.0 as u32;
        let y = coord.1 as u32;
        if self.morton_idx == self.morton_idx_max {
            None
        } else {
            let x1 = x * self.tilesize;
            let y1 = y * self.tilesize;
            // slice image
            let result = (self.img.view(x1, y1, self.tilesize, self.tilesize), x, y);
            self.morton_idx += 1;
            Some(result)
        }
    }
}
