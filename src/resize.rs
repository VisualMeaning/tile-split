use image::imageops;
use image::DynamicImage;
use crate::Config;

pub type ResizedItem = (DynamicImage, u8);

pub struct Resizer<'c> {
    pub config: &'c Config<'c>,
    max_dimention_size: u32,
}

impl<'c> Resizer<'c> {
    fn _check_dimension(&self, img: &'c DynamicImage) {
        let (img_width, img_height) = (img.width(), img.height());
        if img_width != self.max_dimention_size || img_height != self.max_dimention_size {
            panic!("Image of size {w}x{h} cannot be split into
                tiles of size {tile_size} and max zoom level {max_zoom}.",
                w=img_width,
                h=img_height,
                tile_size=self.config.tilesize,
                max_zoom=self.max_dimention_size,
            );
        }
    }

    fn _resize(&self, img: &'c DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize(width, height, imageops::FilterType::Lanczos3)
    }

    pub fn new(config: &'c Config<'c>) -> Self {
        let Config{zoomlevel, zoomrange, tilesize, ..} = &config;
        // Do we need zoomlevel?
        if zoomrange.iter().any(|x| x > zoomlevel) {
            panic!("Zoom range has value(s) larger than zoom level.");
        }
        let max_dimention_size = tilesize << zoomlevel;

        Self {
            config,
            max_dimention_size
        }
    }

    pub fn resize_range(&self, img: &'c DynamicImage) -> Vec<ResizedItem> {
        self._check_dimension(img);
    
        self.config.zoomrange.iter().map(|&x| {
            let t_size = self.config.tilesize << x;
            (self._resize(img, t_size, t_size), x)
        }).collect()
    }
}
