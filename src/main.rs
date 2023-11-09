use tile_split::{Config, TileImage};

// what is main() returning?
fn main() {

    // read $1
    // read envvars
    // make Config
    let config = Config {
        tilesize: 1024,
        filename: "test.png",
        zoomlevel: 5,
        folder: "out",
    };

    std::fs::create_dir_all(&config.folder).unwrap();

    let zoom = config.zoomlevel;
    let tile_image = TileImage{
        config: &config,
    };
    tile_image.iter(&tile_image.create_img().unwrap()).for_each(|(img, x, y)| img.to_image().save(format!("{p}/{z}_{x}_{y}.png", p=config.folder, z=zoom, x = x, y = y)).unwrap());
}
