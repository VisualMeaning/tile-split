use tile_split::{Config, TileImage};

// what is main() returning?
fn main() {

    // read $1
    // read envvars
    // make Config
    let config = Config {
        tilesize: 1024,
        filename: "test.png".to_string(),
        zoomlevel: 5,
        folder: "out".to_string(),
    };

    std::fs::create_dir_all(&config.folder).unwrap();

    let tile_image = TileImage{
        config: config,
    };
    tile_image.iter().save(format!("{p}/{z}_{x}_{y}.png", p=config.folder, z=config.zoomlevel, x = x, y = y))?;
}
