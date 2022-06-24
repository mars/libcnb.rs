//! Integration tests using libcnb-test.
//!
//! All integration tests are skipped by default (using the `ignore` attribute),
//! since performing builds is slow. To run the tests use: `cargo test -- --ignored`

// Enable Clippy lints that are disabled by default.
// https://rust-lang.github.io/rust-clippy/stable/index.html
#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, assert_not_contains, PackResult, TestConfig, TestRunner};
use std::io::{Read, Write};
use std::net;
use std::net::ToSocketAddrs;
use std::time::Duration;
use std::{fs, io};

#[test]
#[ignore]
fn basic() {
    let config = TestConfig::new("heroku/buildpacks:20", "test-fixtures/simple-ruby-app");

    TestRunner::default().run_test(&config, |context| {
        assert_contains!(context.pack_stdout, "---> Ruby Buildpack");
        assert_contains!(context.pack_stdout, "---> Installing bundler");
        assert_contains!(context.pack_stdout, "---> Installing gems");

        context
            .prepare_container()
            .env("PORT", TEST_PORT.to_string())
            .expose_port(TEST_PORT)
            .start_with_default_process(|container| {
                std::thread::sleep(Duration::from_secs(1));

                assert_eq!(
                    call_test_fixture_service(
                        container.address_for_port(TEST_PORT).unwrap(),
                        "Hello World!"
                    )
                    .unwrap(),
                    "!dlroW olleH"
                );

                assert_contains!(
                    container.shell_exec("ruby --version").stdout,
                    "ruby 2.7.0p0"
                );
            });

        context.run_test(&config, |context| {
            assert_not_contains!(context.pack_stdout, "---> Installing bundler");
            assert_not_contains!(context.pack_stdout, "---> Installing gems");
        });
    });
}

#[test]
#[ignore]
fn missing_gemfile_lock() {
    TestRunner::default().run_test(
        TestConfig::new("heroku/buildpacks:20", "test-fixtures/simple-ruby-app")
            .app_dir_preprocessor(|path| fs::remove_file(path.join("Gemfile.lock")).unwrap())
            .expected_pack_result(PackResult::Failure),
        |context| {
            assert_contains!(
                context.pack_stdout,
                "ERROR: No buildpack groups passed detection."
            );
        },
    );
}

fn call_test_fixture_service<A>(a: A, payload: impl AsRef<str>) -> io::Result<String>
where
    A: ToSocketAddrs,
{
    let mut stream = net::TcpStream::connect(a)?;

    stream.write_all(format!("{}\n", payload.as_ref()).as_bytes())?;

    let mut buffer = vec![];
    stream.read_to_end(&mut buffer)?;

    Ok(format!("{}", String::from_utf8_lossy(&buffer)))
}

const TEST_PORT: u16 = 12346;
