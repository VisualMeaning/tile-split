use std::ops::RangeInclusive;

use image::{imageops, DynamicImage};
use rayon::prelude::*;

use crate::TileImage;

pub struct Config {
    pub tilesize: u32, // 256
    pub zoomlevel: u8, // eg 5
    pub parentzoomlevel: Option<u8>,
    pub indexforzoom: u8,
    pub preset: Option<u8>,
    pub zoomrangetoslice: RangeInclusive<u8>,
    pub targetrangetoslice: RangeInclusive<u32>,
}

impl Config {
    pub fn new(
        // $1
        tilesize: u32, // 256
        zoomlevel: u8, // eg 5
        parentzoomlevel: Option<u8>,
        indexforzoom: u8,                         // eg 1
        zoomrange: Option<RangeInclusive<u8>>,    // eg 0 - 5
        targetrange: Option<RangeInclusive<u32>>, //eg 0 - 500
        preset: Option<u8>,
    ) -> Self {
        if let Some(parentzoomlevelvalue) = parentzoomlevel {
            if parentzoomlevelvalue <= zoomlevel {
                panic!(
                    "Parent zoom level number needs to be bigger than the input image zoom level."
                )
            }
            if indexforzoom > (1 << ((parentzoomlevelvalue - zoomlevel) * 2)) - 1 {
                panic!("indexforzoom value is larger than allowed.")
            }
            Config {
                tilesize,
                zoomlevel,
                parentzoomlevel,
                indexforzoom,
                preset,
                zoomrangetoslice: zoomlevel..=zoomlevel,
                targetrangetoslice: 0..=(1 << (zoomlevel * 2)) - 1,
            }
        } else {
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
                parentzoomlevel,
                indexforzoom,
                preset,
                zoomrangetoslice: startzoomrangetoslice..=endzoomrangetoslice,
                targetrangetoslice: starttargetrange..=endtargetrange,
            }
        }
    }

    pub fn resize_range(&self, img: &DynamicImage) -> Vec<(TileImage, u8)> {
        <RangeInclusive<u8> as Clone>::clone(&self.zoomrangetoslice)
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
        let config = Config::new(256, 5, None, 0, None, None, None);
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 0..=1023);
        assert_eq!(config.preset, None);
    }

    #[test]
    // slice all tiles
    fn full_zoom() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(0, 5)),
            None,
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 0..=5);
        assert_eq!(config.targetrangetoslice, 0..=1023);
        assert_eq!(config.preset, Some(2));
    }

    #[test]
    // slice the first 341 tiles out of all tiles
    fn full_zoom_1() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(0, 341)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 0..=4);
        assert_eq!(config.targetrangetoslice, 0..=255);
    }

    #[test]
    // slice the second 341 tiles out of all tiles
    fn full_zoom_2() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(341, 682)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 0..=340);
    }

    #[test]
    // slice the third 341 tiles out of all tiles
    fn full_zoom_3() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(682, 1023)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 341..=681);
    }

    #[test]
    // slice the remaining tiles out of all tiles
    fn full_zoom_4() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(0, 5)),
            Some(RangeInclusive::new(1023, 1365)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 682..=1023);
    }

    #[test]
    // slice the first 448 tiles out of all tiles
    fn half_zoom_1() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(0, 448)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 3..=5);
        assert_eq!(config.targetrangetoslice, 0..=127);
    }

    #[test]
    // slice the second 448 tiles out of all tiles
    fn half_zoom_2() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(448, 896)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 128..=575);
    }

    #[test]
    // slice the remaining tiles out of all tiles
    fn half_zoom_3() {
        let config = Config::new(
            256,
            5,
            None,
            0,
            Some(RangeInclusive::new(3, 5)),
            Some(RangeInclusive::new(896, 1344)),
            Some(2),
        );
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 576..=1023);
    }

    #[test]
    // slice the third level 5 sub image of a level 6 image
    fn sub_image_level_6() {
        let config = Config::new(256, 5, Some(6), 2, None, None, Some(2));
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 0..=1023);
    }

    #[test]
    // slice the 15th level 5 sub image of a level 7 image
    fn sub_image_level_7() {
        let config = Config::new(256, 5, Some(7), 15, None, None, Some(2));
        assert_eq!(config.zoomrangetoslice, 5..=5);
        assert_eq!(config.targetrangetoslice, 0..=1023);
    }

    #[test]
    #[should_panic]
    // should panic if zoomlevel is larger than parentzoomlevel
    fn sub_image_zoom_largerthan_parent() {
        Config::new(256, 5, Some(4), 15, None, None, Some(2));
    }

    #[test]
    #[should_panic]
    // should panic if indexforzoom is larger than allowed
    fn sub_image_wrong_indexforzoom_1() {
        Config::new(256, 5, Some(6), 4, None, None, Some(2));
    }

    #[test]
    #[should_panic]
    // should panic if indexforzoom is larger than allowed
    fn sub_image_wrong_indexforzoom_2() {
        Config::new(256, 5, Some(7), 16, None, None, Some(2));
    }
}
