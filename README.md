# Slicing map tiles from illustrations with Rust

The goal is to recreate the functionality of tiles slicing from `imagemagick` in Rust. The slicing time has shown to be siginificantly reduced, and the built binary can be run as a platform agnostic program.

## Installation for code contribution

Follow the steps from [Rust Documentation](https://doc.rust-lang.org/book/ch01-01-installation.html)

## Building the project

If you are working with the code, run `cargo build` to create a debug build executable file in `target/debug/tile-split`.

If you want to generate a distributable binary, run `cargo build -r` to create a release build executable file in `target/release/tile-split`.

## Running the executable

**Prerequisite for Mac users:** Right click ( control + click ) the `tile-split` program and click Open from the menu. Close the terminal and continue as normal. This step is to create an exception to Mac's security rules for running an unauthorized program. 

### Usage

We give the binary a PNG file as input, and we tell it what zoomlevel that PNG file is via the --zoomlevel arg. Currently it will just slice the whole thing into tiles of --tilesize dimensions. If we want to do a different zoomlevel, we need to give it a different-sized PNG. Using the --zoomrange command to specify which zoomlevels we want tiles for. For example: 

`tile-split level-5.png --zoomlevel 5 --zoomrange 2 3 4 5`

This would generate tiles for zoomlevels 2, 3, 4 and 5 from the single level 5 PNG we give it.

Run `tile-split --help` for more command description.

```
Split input image files into sets of tiles.

Usage: tile-split [OPTIONS] --zoomlevel <ZOOMLEVEL> <FILENAME>

Arguments:
  <FILENAME>  Input PNG filename

Options:
  -l, --zoomlevel <ZOOMLEVEL>     Zoomlevel of input PNG file [env: ZOOMLEVEL=]
  -r, --zoomrange <ZOOMRANGE>...  Zoomrange to slice tiles for, currently unused
  -o, --output-dir <OUTPUT_DIR>   Location to write output tiles to [env: OUTPUT_DIR=] [default: out]
  -s, --tilesize <TILESIZE>       Dimension of output tiles, in pixels [default: 256]
  -f, --tileformat <TILEFORMAT>   Type of output tiles, currently unused [env: TILEFORMAT=] [default: png]
  -h, --help                      Print help
  -V, --version                   Print version
```


