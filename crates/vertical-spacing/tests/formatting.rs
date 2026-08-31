use std::path::PathBuf;

use larvae::fmt::FmtConfig;
use vertical_spacing_worm::format_luau;

fn check_fixture(name: &str, fmt: &FmtConfig) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let input = std::fs::read_to_string(directory.join("input.luau"))
        .expect("the fixture input should be readable");
    let expected = std::fs::read_to_string(directory.join("expected.luau"))
        .expect("the fixture expectation should be readable");
    let actual = format_luau(&input, fmt).expect("the fixture should format");

    assert_eq!(actual, expected, "fixture {name}");
    assert_eq!(
        format_luau(&actual, fmt).expect("formatted output should format again"),
        actual,
        "fixture {name} should be idempotent"
    );
}

#[test]
fn top_level_groups() {
    check_fixture("groups", &FmtConfig::default());
}

#[test]
fn multiline_and_simple_types() {
    let mut fmt = FmtConfig::default();
    fmt.table_types.width = 20;

    check_fixture("types", &fmt);
}

#[test]
fn declarations_and_related_mutations() {
    check_fixture("declarations", &FmtConfig::default());
}

#[test]
fn guards_and_noninitial_returns() {
    check_fixture("guards", &FmtConfig::default());
}

#[test]
fn initial_returns() {
    check_fixture("first-return", &FmtConfig::default());
}
