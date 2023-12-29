use crate::Config;
use image::imageops;
use image::DynamicImage;

pub trait Resizer<'iter, T> {
    type ItemIterator: Iterator<Item = (T, u8)>;

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
    type ItemIterator = Box<dyn Iterator<Item = ResizedItem> + 'iter>;

    fn resize_range(&'iter self, img: &'iter DynamicImage) -> Self::ItemIterator {
        _check_dimension(self, img);

        Box::new(
            (self.startzoomrangetoslice..self.endzoomrangetoslice).map(|x| {
                let t_size = self.tilesize << x;
                (_resize(img, t_size, t_size), x)
            }),
        )
    }
}
