use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn link_removes_zombie() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("zombie.txt");
    let out_zombie = output.child("zombie.txt");

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

    std::fs::remove_file(&in_file).unwrap();

    assert!(!in_file.exists(), "In file exists");
    assert!(out_zombie.is_symlink());
    assert!(!out_zombie.exists(), "Symlink target exists");
    assert_eq!(out_zombie.read_link().unwrap(), in_file.path());

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

    assert!(!out_zombie.is_symlink(), "Zombie link wan't removed!");
    assert!(!out_zombie.exists());

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn unlink_removes_zombie() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("zombie.txt");
    let out_zombie = output.child("zombie.txt");

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

    std::fs::remove_file(&in_file).unwrap();

    assert!(!in_file.exists(), "In file exists");
    assert!(out_zombie.is_symlink());
    assert!(!out_zombie.exists(), "Symlink target exists");
    assert_eq!(out_zombie.read_link().unwrap(), in_file.path());

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

    assert!(!out_zombie.is_symlink(), "Zombie link wan't removed!");
    assert!(!out_zombie.exists());

    package.close().unwrap();
    output.close().unwrap();
}

#[test]
fn remove_zombie_dirs() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    let in_dir = package.child("sub_dir");
    let in_file = in_dir.child("zombie.txt");

    let out_dir = output.child("sub_dir");
    let out_zombie = out_dir.child("zombie.txt");

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

    std::fs::remove_dir_all(&in_dir).unwrap();

    assert!(!in_file.exists(), "In file exists");
    assert!(!in_dir.exists(), "In dir exists");
    assert!(out_dir.exists(), "Out dir doesn't exist");
    assert!(out_zombie.is_symlink(), "Out file isn't a symlink");
    assert!(!out_zombie.exists(), "Symlink target exists");
    assert_eq!(out_zombie.read_link().unwrap(), in_file.path());

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

    assert!(!in_file.exists(), "In file exists");
    assert!(!out_zombie.is_symlink(), "Out file is still a symlink");
    assert!(!out_zombie.exists(), "Symlink target exists");
    assert!(!in_dir.exists(), "In dir exists");
    assert!(!out_dir.exists(), "Out dir still exists");
}

#[test]
fn dont_remove_other_empty_dirs() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    let out_dir = output.child("sub_dir");
    out_dir.create_dir_all().unwrap();

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

    assert!(out_dir.exists(), "Out dir doesn't exist");
}
