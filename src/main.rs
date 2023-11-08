use tile_split::{Config, tile_image};

// what is main() returning?
fn main() {

    // read $1
    // read envvars
    // make Config
    let config = Config {
        tilesize: 256,
        filename: "test.png".to_string(),
        zoomlevel: 5,
        folder: "out".to_string(),
    };

    std::fs::create_dir_all(&config.folder).unwrap();

    tile_image(config).unwrap();
    // return result code?
}
