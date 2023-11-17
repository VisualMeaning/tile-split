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
            morton_idx_max: (img.width() / self.config.tilesize as u32)
                * (img.height() / self.config.tilesize as u32),
            tilesize: self.config.tilesize,
        }
    }
}

pub struct TilesIterator<'d> {
    img: &'d DynamicImage,
    morton_idx: u32,
    morton_idx_max: u32,
    tilesize: u16,
}

impl<'d> Iterator for TilesIterator<'d> {
    type Item = (SubImage<&'d DynamicImage>, u16, u16);
    fn next(&mut self) -> Option<Self::Item> {
        // reaching the end of slicing, return None
        let coord = coord_of(self.morton_idx);
        if self.morton_idx == self.morton_idx_max {
            None
        } else {
            let x1 = coord.0 * self.tilesize;
            let y1 = coord.1 * self.tilesize;
            // slice image
            let result = (
                self.img.view(
                    x1.into(),
                    y1.into(),
                    self.tilesize.into(),
                    self.tilesize.into(),
                ),
                coord.0,
                coord.1,
            );
            self.morton_idx += 1;
            Some(result)
        }
    }
}
