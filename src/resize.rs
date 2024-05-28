use std::ops::RangeInclusive;

use crate::Config;
use image::imageops;
use image::DynamicImage;
use rayon::prelude::*;

pub trait Resizer<'iter, T> {
    type ItemIterator;
    fn resize_range(&'iter self, img: &'iter T) -> Self::ItemIterator;
}

type ResizedItem = (DynamicImage, u8);

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

fn _resize(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize(width, height, imageops::FilterType::Lanczos3)
}

impl<'iter> Resizer<'iter, DynamicImage> for Config<'_> {
    type ItemIterator = rayon::iter::Map<rayon::range_inclusive::Iter<u8>, _>;

    fn resize_range(&'iter self, img: &'iter DynamicImage) -> Self::ItemIterator {
        _check_dimension(self, img);

        fn _test<'iter>(state: (& DynamicImage, u32), x: u8) -> ResizedItem {
            let t_size: u32 = state.1 << x;
            (_resize(state.0, t_size, t_size), x)
        }

        RangeInclusive::new(self.startzoomrangetoslice, self.endzoomrangetoslice)
        .into_par_iter()
        .map_init(|| (img, self.tilesize), _test as fn((& DynamicImage, u32), u8) -> ResizedItem )
    }
}
