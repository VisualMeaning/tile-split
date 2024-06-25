use crate::Config;
use image::GenericImageView;
use image::{DynamicImage, ImageResult, SubImage};
use std::path::Path;
use zorder::coord_of;

pub struct TileImage<'c> {
    pub config: &'c Config,
    pub img: DynamicImage,
}

impl<'c> TileImage<'c> {
    pub fn slice_tiles(&self, index: u8) -> Vec<Tile> {
        let width_in_tiles = self.img.width() / self.config.tilesize;
        let height_in_tiles = self.img.height() / self.config.tilesize;
        let morton_idx_max = width_in_tiles * height_in_tiles;
        let tileimage_coord: (u16, u16) = match &self.config.parentzoomlevel {
            None => (0, 0),
            Some(parentzoomlevel) => {
                if parentzoomlevel > &self.config.zoomlevel {
                    coord_of(self.config.indexforzoom.into())
                } else {
                    (0, 0)
                }
            }
        };
        let output_level = match &self.config.parentzoomlevel {
            None => index,
            Some(parentzoomlevel) => {
                if parentzoomlevel > &self.config.zoomlevel {
                    parentzoomlevel.clone()
                } else {
                    index
                }
            }
        };

        let mut targetrangetoslice = 0..=morton_idx_max - 1;
        // if startzoomrangetoslice is the same as endzoomrangetoslice,
        // then tiles to be sliced in this function are from same zoom level
        if self.config.zoomrangetoslice.start() == self.config.zoomrangetoslice.end() {
            if index == *self.config.zoomrangetoslice.end() {
                targetrangetoslice =
                    *self.config.targetrangetoslice.start()..=*self.config.targetrangetoslice.end();
            }
        // otherwise, the start zoom level should slice tiles from starttargetrange to end,
        // the end zoom level should slice tiles from 0 to endtargetrange
        } else if index == *self.config.zoomrangetoslice.start() {
            if index > 0 {
                targetrangetoslice =
                    *self.config.targetrangetoslice.start()..=(1 << (index * 2)) - 1;
            }
        } else if index == *self.config.zoomrangetoslice.end() {
            targetrangetoslice = 0..=*self.config.targetrangetoslice.end();
        }

        targetrangetoslice
            .map(|morton_idx| {
                let coord = coord_of(morton_idx);
                let tilename = format!(
                    "{z}-{x}-{y}",
                    z = output_level,
                    x = (coord.0 + (tileimage_coord.0 * 32)) as u32,
                    y = (coord.1 + (tileimage_coord.1 * 32)) as u32
                );
                Tile {
                    config: self.config,
                    parent_img: &self.img,
                    x: coord.0 as u32,
                    y: coord.1 as u32,
                    name: tilename,
                }
            })
            .collect()
    }

    pub fn save_image(&self, z: u8, folder: &Path, tileformat: &str) -> ImageResult<()> {
        self.img
            .save(folder.join(format!("{z}.{fmt}", z = z, fmt = tileformat)))
    }
}

pub struct Tile<'c> {
    pub config: &'c Config,
    pub parent_img: &'c DynamicImage,
    pub x: u32,
    pub y: u32,
    pub name: String,
}

impl<'c> Tile<'c> {
    pub fn to_subimage(&self) -> SubImage<&DynamicImage> {
        let ts = self.config.tilesize;
        self.parent_img.view(self.x * ts, self.y * ts, ts, ts)
    }

    pub fn convert_to_oxipng(&self, img: SubImage<&DynamicImage>) -> Vec<u8> {
        let ts = self.config.tilesize;
        oxipng::RawImage::new(
            ts,
            ts,
            oxipng::ColorType::RGBA,
            oxipng::BitDepth::Eight,
            img.to_image().into_raw(),
        )
        .unwrap()
        .create_optimized_png(&oxipng::Options::from_preset(self.config.preset.unwrap()))
        .unwrap()
    }
}
