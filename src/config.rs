

pub struct Config<'a> {
    pub filename: &'a str, // $1
    pub tilesize: u32, // 256
    pub zoomlevel: u8, // eg 0 - 5
    // don't need MGK
    pub folder: &'a str, // $OUTDIR out
}
