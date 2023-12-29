use std::ops::RangeInclusive;
use std::path::Path;

pub struct Config<'a> {
    pub filename: &'a Path, // $1
    pub tilesize: u32,      // 256
    pub zoomlevel: u8,      // eg 5
    pub startzoomrangetoslice: u8,
    pub endzoomrangetoslice: u8,
    pub starttargetrange: u32,
    pub endtargetrange: u32,
}

impl<'a> Config<'a> {
    pub fn new(
        filename: &'a Path,            // $1
        tilesize: u32,                 // 256
        zoomlevel: u8,                 // eg 5
        zoomrange: RangeInclusive<u8>, // eg 0 - 5
        functionindex: u32,            // eg 2
        totalfunction: u32,            // eg 4
    ) -> Self {
        let zomr = if zoomrange.is_empty() {
            zoomlevel..=zoomlevel
        } else {
            zoomrange
        };
        // total number of tiles required in zoomrange
        let mut totaltiles = 0;
        zomr.clone().for_each(|x| {
            totaltiles += 1 << (x * 2);
        });
        let average = totaltiles / totalfunction;
        // number of tiles sliced by previous functions
        let tilessliced = average * (functionindex - 1);
        // number of tiles to slice in this function, if it's the last
        // function, it should slice all the remaining tiles
        let mut tilestoslice = average;
        if functionindex == totalfunction {
            tilestoslice = totaltiles - tilessliced;
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
            filename,
            zoomlevel,
            startzoomrangetoslice,
            endzoomrangetoslice,
            starttargetrange,
            endtargetrange,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::{ops::RangeInclusive, path::Path};

    #[test]
    // one function in total
    fn full_zoom() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(0, 5),
            1,
            1,
        );
        assert_eq!(config.startzoomrangetoslice, 0);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
    }

    #[test]
    // the first function out of 4 functions
    fn full_zoom_1() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(0, 5),
            1,
            4,
        );
        assert_eq!(config.startzoomrangetoslice, 0);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 4);
        assert_eq!(config.endtargetrange, 255);
    }

    #[test]
    // the second function out of 4 functions
    fn full_zoom_2() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(0, 5),
            2,
            4,
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 340);
    }

    #[test]
    // the third function out of 4 functions
    fn full_zoom_3() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(0, 5),
            3,
            4,
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 341);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 681);
    }

    #[test]
    // the fourth function out of 4 functions
    fn full_zoom_4() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(0, 5),
            4,
            4,
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 682);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
    }

    #[test]
    // the first function out of 3 functions
    fn half_zoom_1() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(3, 5),
            1,
            3,
        );
        assert_eq!(config.startzoomrangetoslice, 3);
        assert_eq!(config.starttargetrange, 0);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 127);
    }

    #[test]
    // the second function out of 3 functions
    fn half_zoom_2() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(3, 5),
            2,
            3,
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 128);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 575);
    }

    #[test]
    // the third function out of 3 functions
    fn half_zoom_3() {
        let config = Config::new(
            &Path::new("test.png"),
            256,
            5,
            RangeInclusive::new(3, 5),
            3,
            3,
        );
        assert_eq!(config.startzoomrangetoslice, 5);
        assert_eq!(config.starttargetrange, 576);
        assert_eq!(config.endzoomrangetoslice, 5);
        assert_eq!(config.endtargetrange, 1023);
    }
}
