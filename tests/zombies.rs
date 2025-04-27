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
fn remove_1_zombie_dir() {
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
fn remove_zombie_nested_dirs() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    let in_dir = package.child("a");
    let in_dir2 = in_dir.child("b");
    let in_file = in_dir2.child("c.txt");

    let out_dir = output.child("a");
    let out_dir2 = out_dir.child("b");
    let out_zombie = out_dir2.child("c.txt");

    in_file.touch().unwrap();

    assert!(in_dir.exists(), "In dir wasn't created");
    assert!(in_dir2.exists(), "In dir2 wasn't created");
    assert!(in_file.exists(), "In file wasn't created");

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
fn dont_remove_nested_dirs_with_zombies() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();

    let in_dir = package.child("a");
    let in_file = in_dir.child("a_a.txt");
    let in_dir2 = in_dir.child("b");
    let in_file2 = in_dir2.child("b_a.txt");

    let out_dir = output.child("a");
    let out_file = out_dir.child("a_a.txt");
    let out_dir2 = out_dir.child("b");
    let out_file2 = out_dir2.child("b_a.txt");

    in_file.touch().unwrap();
    in_file2.touch().unwrap();

    assert!(in_dir.exists(), "In dir wasn't created");
    assert!(in_dir2.exists(), "In dir2 wasn't created");
    assert!(in_file.exists(), "In file wasn't created");
    assert!(in_file2.exists(), "In file2 wasn't created");

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

    std::fs::remove_dir_all(&in_dir2).unwrap();

    assert!(in_file.exists(), "In file doesn't exist");
    assert!(!in_file2.exists(), "In file2 exists");
    assert!(in_dir.exists(), "In dir doesn't exist");
    assert!(!in_dir2.exists(), "In dir2 exists");
    assert!(out_dir.exists(), "Out dir doesn't exist");
    assert!(out_file.is_symlink(), "Out file isn't a symlink");
    assert!(out_file.exists(), "Out file symlink target doesn't exist");
    assert_eq!(out_file.read_link().unwrap(), in_file.path());
    assert!(out_file2.is_symlink(), "Out file isn't a symlink");
    assert!(!out_file2.exists(), "Out file2 target exists");
    assert_eq!(out_file2.read_link().unwrap(), in_file2.path());

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
    assert!(out_file.is_symlink(), "Out file isn't a symlink");
    assert!(out_file.exists(), "Out file link target doesn't exist");
    assert!(!out_file2.is_symlink(), "Out file2 link wasn't removed");
    assert!(!out_file2.exists(), "Out file2 link target exists");
    assert!(in_dir.exists(), "In dir doesn't exist");
    assert!(out_dir.exists(), "Out dir doesn't exist");
    assert!(!out_dir2.exists(), "Out dir2 exists");
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
