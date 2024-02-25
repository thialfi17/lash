use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn link_1_file() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert().success();

    assert!(out_file.is_symlink());
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn unlink_1_file() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();
    out_file.symlink_to_file(in_file.path()).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert().success();

    assert_eq!(out_file.exists(), false);

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file_dry_run() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--dry-run",
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert().success();

    assert_eq!(out_file.exists(), false);

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn unlink_1_file_dry_run() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();
    out_file.symlink_to_file(in_file.path()).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--dry-run",
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert().success();

    assert!(out_file.is_symlink());
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    package.close().unwrap();
    output.close().unwrap();
}
