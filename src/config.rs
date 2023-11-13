
pub struct Config<'a> {
    pub filename: &'a str, // $1
    pub tilesize: u32, // 256
    pub zoomlevel: u8, // eg 0 - 5
    pub zoomrange: &'a Vec<u8>, // eg 0 - 5
    pub tileformat: &'a str, // png
    pub folder: &'a str, // $OUTDIR out
}
