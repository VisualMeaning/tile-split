use std::env;
use tile_split::{Config, TileImage};

// what is main() returning?
fn main() {
    // Exit with an error if there is no filename arg
    let filename: String = match env::args().nth(1) {
        Some(arg) => arg.to_string(),
        None => {
            eprintln!("Error: Please provide a filename.");
            std::process::exit(1);
        }
    };
    let folder: String = env::var("OUTDIR").unwrap_or("out".to_string());
    let format: String = env::var("FMT").unwrap_or("png".to_string());
    let zoomlevel: u8 = env::var("ZOOMLEVEL").map(|s| s.parse::<u8>()).unwrap_or(Ok(5)).unwrap_or(5);

    let config: Config = Config {
        filename,
        folder,
        format,
        tilesize: 256,
        zoomlevel,
    };

    std::fs::create_dir_all(&config.folder).unwrap();

    let zoom = config.zoomlevel;
    let tile_image = TileImage{
        config: &config,
    };
    tile_image.iter(&tile_image.create_img().unwrap()).for_each(|(img, x, y)| img.to_image().save(format!("{p}/{z}_{x}_{y}.png", p=config.folder, z=zoom, x = x, y = y)).unwrap());
}
