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
    let args = parse(["bin", "-l", "7", "a-file.png", "-r", "0-5",]);
    assert_eq!(args.filename.as_path().display().to_string(), "a-file.png");
    assert_eq!(args.zoomlevel, 7);
    assert_eq!(args.zoomrange, 0..=5);
}
