use snapbox::cmd::Command;

#[test]
fn snapbox() {
    compile_fail(
        "arguments",
        "\
error: `#[methodify]` does not accept arguments
 --> src/lib.rs:3:1
  |
3 | #[methodify(Executable)]
  | ^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this error originates in the attribute macro `methodify` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `methodify-arguments-fixture` (lib) due to 1 previous error
",
    );

    compile_fail(
        "no_arguments",
        "\
error: `#[methodify]` requires at least one function argument
 --> src/lib.rs:4:4
  |
4 | fn no_arguments() {}
  |    ^^^^^^^^^^^^

error: could not compile `methodify-no-arguments-fixture` (lib) due to 1 previous error
",
    );
}

fn compile_fail(fixture: &str, stderr: &str) {
    Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(format!("tests/fixtures/{fixture}/Cargo.toml"))
        .env("CARGO_TARGET_DIR", format!("target/snapbox/{fixture}"))
        .assert()
        .failure()
        .stderr_eq(stderr);
}
