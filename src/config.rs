use std::ops::RangeInclusive;
use std::path::Path;

pub struct Config<'a> {
    pub filename: &'a Path,                // $1
    pub tilesize: u32,                     // 256
    pub zoomlevel: u8,                     // eg 5
    pub zoomrange: RangeInclusive<u8>,     // eg 0 - 5
    pub target_range: RangeInclusive<u32>, //eg 0 - 500
    pub tileformat: &'a str,               // png
    pub folder: &'a Path,                  // $OUTDIR out
}
