use crate::{Config, Error};
use image::{imageops, GenericImageView};
use image::{io::Reader as ImageReader, DynamicImage, SubImage};
use std::ops::RangeInclusive;
use zorder::coord_of;

pub struct TileImage<'c> {
    pub config: &'c Config<'c>,
}

impl<'c> TileImage<'c> {
    pub fn open_img(&self) -> Result<DynamicImage, Error> {
        let mut reader = ImageReader::open(self.config.filename)?;
        // Default memory limit of 512MB is too small for level 6+ PNGs
        reader.no_limits();

        let img = reader.decode()?;

        // TODO: Refactor the image load to be done separately,
        // so the check and error can happen in a function that receives config and an already loaded image
        if img.width() != img.height() {
            return Err("Image is not square.".into());
        }
        Ok(img)
    }

    pub fn iter<'d>(
        &self,
        img: &'d DynamicImage,
        targetrangetoslice: Option<RangeInclusive<u32>>,
    ) -> TilesIterator<'d> {
        let width_in_tiles = img.width() / self.config.tilesize;
        let height_in_tiles = img.height() / self.config.tilesize;
        let morton_idx_max = width_in_tiles * height_in_tiles;

        let morton_idx = match &targetrangetoslice {
            Some(targetrange) => *targetrange.start(),
            None => 0,
        };

        TilesIterator {
            img,
            morton_idx,
            morton_idx_max,
            tilesize: self.config.tilesize,
            targetrange: targetrangetoslice.clone(),
        }
    }

    fn _check_dimension(config: &Config, img: &DynamicImage) {
        if config.endzoomrangetoslice > config.zoomlevel {
            panic!("Zoom range has value(s) larger than zoom level.");
        }
        let (img_width, img_height) = (img.width(), img.height());
        let max_dimension_size = config.tilesize << config.zoomlevel;
        if img_width != max_dimension_size || img_height != max_dimension_size {
            panic!(
                "Image of size {w}x{h} cannot be split into
                tiles of size {tile_size} and max zoom level {max_zoom}.",
                w = img_width,
                h = img_height,
                tile_size = config.tilesize,
                max_zoom = config.zoomlevel,
            );
        }
    }
    
    pub fn resize(&self, img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        Self::_check_dimension(self.config, img);
        img.resize(width, height, imageops::FilterType::Lanczos3)
    }
}

pub struct TilesIterator<'d> {
    img: &'d DynamicImage,
    morton_idx: u32,
    morton_idx_max: u32,
    tilesize: u32,
    targetrange: Option<RangeInclusive<u32>>,
}

impl<'d> Iterator for TilesIterator<'d> {
    type Item = (SubImage<&'d DynamicImage>, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        // Reaching the end of slicing, return None
        let coord = coord_of(self.morton_idx);
        let x = coord.0 as u32;
        let y = coord.1 as u32;
        match &self.targetrange {
            Some(targetrange) if !targetrange.contains(&self.morton_idx) => None,
            None if self.morton_idx == self.morton_idx_max => None,
            _ => {
                let x1 = x * self.tilesize;
                let y1 = y * self.tilesize;
                // Slice image
                let result = (self.img.view(x1, y1, self.tilesize, self.tilesize), x, y);
                self.morton_idx += 1;
                Some(result)
            }
        }
    }
}
