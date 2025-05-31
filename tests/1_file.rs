use std::fs;

use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn link_1_file_no_overwrite() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");
    let out_contents: Vec<u8> = rand::random_iter().take(256).collect();

    in_file.touch().unwrap();
    out_file.write_binary(&out_contents).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    let contents: Vec<u8> = fs::read(in_file.path()).unwrap();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!in_file.is_symlink(), "In file is a symlink");
    assert!(out_file.exists(), "Out file does not exist");
    assert!(!out_file.is_symlink(), "Out file was changed to a symlink");
    assert!(contents.is_empty(), "In file contents were overwritten");

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(out_file.is_symlink());
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
/// For now make sure we don't change anything if there is an intermediate link that eventually
/// resolves to the correct file. This can avoid leaving unneeded links lying around on the
/// filesystem.
///
/// TODO: Is this behaviour necessary and should it have a test i.e should it be codified?
fn link_1_file_nested_symlinks() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");
    let mid_file = output.child("mid");

    in_file.touch().unwrap();
    mid_file.symlink_to_file(&in_file).unwrap();
    out_file.symlink_to_file(&mid_file).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        // Should succeed because output still links into package
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(out_file.exists(), "Out file doesn't exist");
    assert!(out_file.is_symlink(), "Out file isn't a symlink");
    assert!(mid_file.is_symlink(), "Mid file isn't a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        mid_file.path(),
        "out file doesn't point to mid file"
    );
    assert_eq!(
        mid_file.read_link().unwrap(),
        in_file.path(),
        "mid file doesn't point to in file"
    );

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
/// Test that relative symlinks are converted to absolute paths
fn link_1_file_relative_symlinks() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.touch().unwrap();

    let mut relative_path = std::ffi::OsString::from("/.");
    relative_path.push(in_file.as_os_str());

    out_file.symlink_to_file(relative_path).unwrap();

    assert_ne!(
        out_file.read_link().unwrap().as_os_str(),
        out_file
            .read_link()
            .unwrap()
            .canonicalize()
            .unwrap()
            .as_os_str(),
        "out file is already an absolute link"
    );
    assert_eq!(
        out_file.read_link().unwrap().canonicalize().unwrap(),
        in_file.path(),
        "out file doesn't point to in file"
    );

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        // Should succeed because output still links into package
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(out_file.exists(), "Out file doesn't exist");
    assert!(out_file.is_symlink(), "Out file isn't a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "out file doesn't point to in file"
    );
    assert_eq!(
        out_file.read_link().unwrap().as_os_str(),
        out_file
            .read_link()
            .unwrap()
            .canonicalize()
            .unwrap()
            .as_os_str(),
        "out file is not an absolute link"
    );

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
        .current_dir(package.path())
        .args([
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!out_file.exists());

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
        .current_dir(package.path())
        .args([
            "--dry-run",
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!out_file.exists());

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file_adopt() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");
    let out_contents: Vec<u8> = rand::random_iter().take(256).collect();

    in_file.touch().unwrap();
    out_file.write_binary(&out_contents).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    let in_contents = fs::read(in_file.path()).unwrap();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!in_file.is_symlink(), "In file is a symlink");
    assert!(out_file.exists(), "Out file doesn't exist");
    assert!(out_file.is_symlink(), "Out file is not a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );
    assert_eq!(
        in_contents, out_contents,
        "In file hasn't got out file's contents"
    );

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn link_1_file_adopt_dry_run() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");
    let out_contents: Vec<u8> = rand::random_iter().take(256).collect();

    in_file.touch().unwrap();
    out_file.write_binary(&out_contents).unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(package.path())
        .args([
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

    let contents: Vec<u8> = fs::read(in_file.path()).unwrap();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!in_file.is_symlink(), "In file is a symlink");
    assert!(out_file.exists(), "Out file does not exist");
    assert!(!out_file.is_symlink(), "Out file was changed to a symlink");
    assert!(contents.is_empty(), "In file contents were overwritten");

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
        .current_dir(package.path())
        .args([
            "--dry-run",
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(out_file.is_symlink());
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    package.close().unwrap();
    output.close().unwrap();
}
