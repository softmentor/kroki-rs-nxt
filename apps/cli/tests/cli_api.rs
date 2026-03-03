use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn resource_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join(name)
}

#[test]
fn cli_command_is_well_formed() {
    kroki_cli::command().debug_assert();
}

#[test]
fn test_fixture_cli_args_is_present() {
    let args =
        fs::read_to_string(fixture_path("cli-args.txt")).expect("cli fixture args should exist");
    assert!(args.contains("serve"));
}

#[test]
fn test_resource_expected_binary_name_is_present() {
    let binary_name = fs::read_to_string(resource_path("expected-binary-name.txt"))
        .expect("cli expected-binary-name resource should exist");
    assert_eq!(binary_name.trim(), "kroki");
}
