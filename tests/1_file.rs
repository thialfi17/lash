use std::fs;

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
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
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
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
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
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
    assert_eq!(out_file.exists(), false);

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file_adopt() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();
    out_file.write_str("test").unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    let contents = fs::read_to_string(in_file.path()).unwrap();

    assert!(in_file.exists(), "In file exists");
    assert_eq!(in_file.is_symlink(), false, "In file is not a symlink");
    assert_eq!(out_file.exists(), true, "Out file exists");
    assert_eq!(out_file.is_symlink(), true, "Out file is a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );
    assert_eq!("test", contents, "In file has now got out file's contents");

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file_adopt_dry_run() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();
    out_file.write_str("test").unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--verbose",
            "--dry-run",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    let contents = fs::read_to_string(in_file.path()).unwrap();

    assert!(in_file.exists(), "In file exists");
    assert_eq!(in_file.is_symlink(), false, "In file is not a symlink");
    assert_eq!(out_file.exists(), true, "Out file exists");
    assert_eq!(out_file.is_symlink(), false, "Out file is not a symlink");
    assert_eq!("", contents, "In file has still got empty contents");

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
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
    assert!(out_file.is_symlink());
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    package.close().unwrap();
    output.close().unwrap();
}
