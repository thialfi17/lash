use assert_cmd::Command;

#[test]
fn no_args_print_usage() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .assert()
        .code(2);
}

#[test]
fn link_all_options_long() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--dotfiles",
            "--dry-run",
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_all_options_short() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "-n",
            "-v",
            "-t",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn unlink_all_options_long() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--dotfiles",
            "--dry-run",
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn unlink_all_options_short() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "-n",
            "-v",
            "-t",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    package.close().unwrap();
    output.close().unwrap();
}
