use crate::Config;
use image::{imageops, GenericImageView};
use image::{io::Reader as ImageReader, DynamicImage, ImageResult, SubImage};
use std::ops::RangeInclusive;
use std::path::Path;
use zorder::coord_of;

pub struct TileImage<'c> {
    pub config: &'c Config<'c>,
    pub img: DynamicImage,
}

impl<'c> TileImage<'c> {
    pub fn iter_tiles(&self, index: u8) -> TilesIterator<'_> {
        let width_in_tiles = self.img.width() / self.config.tilesize;
        let height_in_tiles = self.img.height() / self.config.tilesize;
        let morton_idx_max = width_in_tiles * height_in_tiles;

        let mut targetrangetoslice: Option<RangeInclusive<u32>> = None;
        // if startzoomrangetoslice is the same as endzoomrangetoslice,
        // then tiles to be sliced in this function are from same zoom level
        if self.config.startzoomrangetoslice == self.config.endzoomrangetoslice {
            if index == self.config.endzoomrangetoslice {
                targetrangetoslice =
                    Some(self.config.starttargetrange..=self.config.endtargetrange);
            }
        // otherwise, the start zoom level should slice tiles from starttargetrange to end,
        // the end zoom level should slice tiles from 0 to endtargetrange
        } else if index == self.config.startzoomrangetoslice {
            if 1 << (index * 2) > 1 {
                targetrangetoslice = Some(self.config.starttargetrange..=(1 << (index * 2)) - 1);
            }
        } else if index == self.config.endzoomrangetoslice {
            targetrangetoslice = Some(0..=self.config.endtargetrange);
        }

        let morton_idx = match &targetrangetoslice {
            Some(targetrange) => *targetrange.start(),
            None => 0,
        };

        TilesIterator {
            img: &self.img,
            config: self.config,
            morton_idx,
            morton_idx_max,
            tilesize: self.config.tilesize,
            targetrange: targetrangetoslice.clone(),
        }
    }

    fn _check_dimension(&self) {
        // TODO: work with any dimension (albeit square image),
        // resize to proper zoom size then split into tiles of config.tilesize side.
        if self.config.endzoomrangetoslice > self.config.zoomlevel {
            panic!("Zoom range has value(s) larger than zoom level.");
        }
        let (img_width, img_height) = (self.img.width(), self.img.height());
        let max_dimension_size = self.config.tilesize << self.config.zoomlevel;
        if img_width != max_dimension_size || img_height != max_dimension_size {
            panic!(
                "Image of size {w}x{h} cannot be split into
                tiles of size {tile_size} and max zoom level {max_zoom}.",
                w = img_width,
                h = img_height,
                tile_size = self.config.tilesize,
                max_zoom = self.config.zoomlevel,
            );
        }
    }

    pub fn save_image(&self, z: u8, folder: &Path, tileformat: &str) -> ImageResult<()> {
        self.img
            .save(folder.join(format!("{z}.{fmt}", z = z, fmt = tileformat)))
    }
}

pub struct TilesIterator<'d> {
    img: &'d DynamicImage,
    config: &'d Config<'d>,
    morton_idx: u32,
    morton_idx_max: u32,
    tilesize: u32,
    targetrange: Option<RangeInclusive<u32>>,
}

impl<'d> Iterator for TilesIterator<'d> {
    type Item = Tile<'d>;
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
                self.morton_idx += 1;
                // Slice image
                Some(Tile {
                    config: self.config,
                    img: self.img.view(x1, y1, self.tilesize, self.tilesize),
                    x,
                    y,
                })
            }
        }
    }
}

pub struct Tile<'c> {
    pub config: &'c Config<'c>,
    pub img: SubImage<&'c DynamicImage>,
    pub x: u32,
    pub y: u32,
}

impl<'c> Tile<'c> {
    pub fn convert_to_oxipng(&self) -> Vec<u8> {
        oxipng::RawImage::new(
            self.config.tilesize,
            self.config.tilesize,
            oxipng::ColorType::RGBA,
            oxipng::BitDepth::Eight,
            self.img.to_image().into_raw(),
        )
        .unwrap()
        .create_optimized_png(&oxipng::Options::from_preset(self.config.preset.unwrap()))
        .unwrap()
    }
}
