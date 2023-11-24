use super::*;

/// Parse args and panic on failure but without exiting the process
fn parse<'a, I>(arr: I) -> Args
where
    I: std::iter::IntoIterator<Item = &'a str>,
{
    Args::try_parse_from(arr).expect("arg parse failed")
}

#[test]
fn args_prompt_help_on_empty() {
    let err = Args::try_parse_from(["bin"]).expect_err("arg parse passed!?");
    let out = err.to_string();
    assert!(out.contains("--help"), "need --help in output {:?}", out);
}

#[test]
#[should_panic] // known failure, --zoomlevel is required
fn args_file_only() {
    let args = parse(["bin", "a-file.png"]);
    assert_eq!(args.filename.as_path().display().to_string(), "a-file.png");
}

#[test]
fn all_args() {
    let args = parse(["bin", "-l", "7", "a-file.png", "-r", "0-5", "-t", "0-333"]);
    assert_eq!(args.filename.as_path().display().to_string(), "a-file.png");
    assert_eq!(args.zoomlevel, 7);
    assert_eq!(args.zoomrange, 0..=5);
    assert_eq!(args.targetrange, Some(0..=333));
}

#[test]
fn open_img_non_square() {
    let img_data: Vec<u8> = vec![255; 300 * 200 * 3]; // RGB image 300 x 200
    let config = Config {
        filename: &PathBuf::from(String::from_utf8_lossy(&img_data).to_string()),
        tilesize: 50,
        targetrange: None,
        zoomlevel: 6,
        zoomrange: 0..=5,
        folder: &PathBuf::from("out"),
        tileformat: "png",
    };
    let tile_image = TileImage { config: &config };
    assert!(tile_image.open_img().is_err());
}
