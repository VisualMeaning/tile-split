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
            None => (1, 1),
            Some(parentzoomlevel) => {
                if parentzoomlevel > &self.config.zoomlevel {
                    coord_of(self.config.indexforzoom.into())
                } else {
                    (1, 1)
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
        if self.config.startzoomrangetoslice == self.config.endzoomrangetoslice {
            if index == self.config.endzoomrangetoslice {
                targetrangetoslice = self.config.starttargetrange..=self.config.endtargetrange;
            }
        // otherwise, the start zoom level should slice tiles from starttargetrange to end,
        // the end zoom level should slice tiles from 0 to endtargetrange
        } else if index == self.config.startzoomrangetoslice {
            if index > 0 {
                targetrangetoslice = self.config.starttargetrange..=(1 << (index * 2)) - 1;
            }
        } else if index == self.config.endzoomrangetoslice {
            targetrangetoslice = 0..=self.config.endtargetrange;
        }

        targetrangetoslice
            .map(|morton_idx| {
                let coord = coord_of(morton_idx);
                let tilename = format!(
                    "{z}-{x}-{y}",
                    z = output_level,
                    x = (coord.0 * tileimage_coord.0) as u32,
                    y = (coord.1 * tileimage_coord.1) as u32
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
