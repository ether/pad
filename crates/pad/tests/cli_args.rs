use clap::Parser;
use pad::cli::{Args, Mode};

#[test]
fn no_args_is_untitled() {
    let a = Args::parse_from(["pad"]);
    assert!(matches!(a.mode(), Mode::Untitled));
}

#[test]
fn one_path_opens_file() {
    let a = Args::parse_from(["pad", "foo.md"]);
    match a.mode() {
        Mode::OpenFile(p) => assert_eq!(p.to_str().unwrap(), "foo.md"),
        m => panic!("expected OpenFile, got {m:?}"),
    }
}

#[test]
fn recover_flag() {
    let a = Args::parse_from(["pad", "--recover"]);
    assert!(matches!(a.mode(), Mode::Recover));
}

#[test]
fn extra_args_rejected() {
    let r = Args::try_parse_from(["pad", "a.txt", "b.txt"]);
    assert!(r.is_err());
}
