use std::ops::RangeInclusive;

use image::{imageops, DynamicImage};
use rayon::prelude::*;

use crate::TileImage;

pub struct Config {
    pub tilesize: u32, // 256
    pub zoomlevel: u8, // eg 5
    pub preset: Option<u8>,
    pub startzoomrangetoslice: u8,
    pub endzoomrangetoslice: u8,
    pub starttargetrange: u32,
    pub endtargetrange: u32,
}

impl Config {
    pub fn new(
        // $1
        tilesize: u32,                            // 256
        zoomlevel: u8,                            // eg 5
        zoomrange: Option<RangeInclusive<u8>>,    // eg 0 - 5
        targetrange: Option<RangeInclusive<u32>>, //eg 0 - 500
        preset: Option<u8>,
    ) -> Self {
        let zomr = zoomrange.unwrap_or(zoomlevel..=zoomlevel);
        // total number of tiles required in zoomrange
        let mut totaltiles = 0;
        zomr.clone().for_each(|x| {
            totaltiles += 1 << (x * 2);
        });
        // number of tiles sliced
        let tilessliced = match &targetrange {
            Some(targetrange) => *targetrange.start(),
            None => 0,
        };
        // number of tiles to slice
        let tilestoslice = match &targetrange {
            Some(targetrange) => *targetrange.end() - tilessliced,
            None => totaltiles,
        };
        if tilestoslice > totaltiles {
            panic!("Target range value cannot be over than the total number of tiles within zoom range.");
        }

        // zoom level to start
        let mut startzoomrangetoslice: u8 = *zomr.clone().start();
        // tile index to start
        let mut starttargetrange: u32 = 0;
        // zoom level to stop
        let mut endzoomrangetoslice: u8 = *zomr.clone().end();
        // tile index to stop
        let mut endtargetrange: u32 = 0;

        // total of tiles in previous zoom levels
        let mut tilessum = 0;
        // calculte startzoomrangetoslice and starttargetrange
        for i in zomr.clone() {
            // number of tiles in this zoom level
            let currentzoomtiles = 1 << (i * 2);
            if tilessum + currentzoomtiles > tilessliced {
                startzoomrangetoslice = i;
                starttargetrange = tilessliced - tilessum;
                break;
            } else {
                tilessum += currentzoomtiles;
            }
        }
        tilessum = 0;
        // calculte endzoomrangetoslice and endtargetrange
        for i in zomr.clone() {
            // number of tiles in this zoom level
            let currentzoomtiles = 1 << (i * 2);
            if tilessum + currentzoomtiles >= tilessliced + tilestoslice {
                endzoomrangetoslice = i;
                endtargetrange = tilessliced + tilestoslice - tilessum - 1;
                break;
            } else {
                tilessum += currentzoomtiles;
            }
        }

        Config {
            tilesize,
            zoomlevel,
            preset,
            startzoomrangetoslice,
            endzoomrangetoslice,
            starttargetrange,
            endtargetrange,
        }
    }

    fn _check_img_dimension(&self, img: &DynamicImage) {
        // TODO: work with any dimension (albeit square image),
        // resize to proper zoom size then split into tiles of config.tilesize side.
        if self.endzoomrangetoslice > self.zoomlevel {
            panic!("Zoom range has value(s) larger than zoom level.");
        }
        let (img_width, img_height) = (img.width(), img.height());
        let max_dimension_size = self.tilesize << self.zoomlevel;
        if img_width != max_dimension_size || img_height != max_dimension_size {
            panic!(
                "Image of size {w}x{h} cannot be split into
                tiles of size {tile_size} and max zoom level {max_zoom}.",
                w = img_width,
                h = img_height,
                tile_size = self.tilesize,
                max_zoom = self.zoomlevel,
            );
        }
    }

    pub fn resize_range(&self, img: &DynamicImage) -> Vec<(TileImage, u8)> {
        self._check_img_dimension(img);
        (self.startzoomrangetoslice..=self.endzoomrangetoslice)
            .into_par_iter()
            .map(|x: u8| {
                let t_size = self.tilesize << x;
                let resized_img = TileImage {
                    config: self,
                    img: img.resize(t_size, t_size, imageops::FilterType::Lanczos3),
                };
                (resized_img, x)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::ops::RangeInclusive;

    #[test]
    // build config with only required args
    fn minimum_args() {
        let config = Config::new(256, 5, None, None, None);
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
        assert_eq!(config.preset, None);
    }

    #[test]
    // slice all tiles
    fn full_zoom() {
        let config = Config::new(256, 5, Some(RangeInclusive::new(0, 5)), None, Some(2));
        assert_eq!(config.startzoomrangetoslice, 0);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
        assert_eq!(config.preset, Some(2));
    }

    #[test]
    // slice the first 341 tiles out of all tiles
    fn full_zoom_1() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(0, 341)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 0);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 4);
        assert_eq!(config.endtargetrange, 255);
    }

    #[test]
    // slice the second 341 tiles out of all tiles
    fn full_zoom_2() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(341, 682)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 340);
    }

    #[test]
    // slice the third 341 tiles out of all tiles
    fn full_zoom_3() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(682, 1023)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 341);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 681);
    }

    #[test]
    // slice the remaining tiles out of all tiles
    fn full_zoom_4() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(1023, 1365)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 682);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
    }

    #[test]
    // slice the first 448 tiles out of all tiles
    fn half_zoom_1() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(0, 448)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 3);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 127);
    }

    #[test]
    // slice the second 448 tiles out of all tiles
    fn half_zoom_2() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(448, 896)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 128);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 575);
    }

    #[test]
    // slice the remaining tiles out of all tiles
    fn half_zoom_3() {
        let config = Config::new(
            256,
            5,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(896, 1344)),
            Some(2),
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 576);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
    }
}
