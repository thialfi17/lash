use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn file_in_directory_created() {
    let package = assert_fs::TempDir::new().unwrap();

    let in_dir = package.child("sub_dir");
    let in_file = in_dir.child("file.txt");

    in_file.touch().unwrap();

    let output = assert_fs::TempDir::new().unwrap();
    let out_dir = output.child("sub_dir");
    let out_file = output.child("sub_dir/file.txt");

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args([
            "--target",
            output.to_str().unwrap(),
            "link",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
    assert!(out_dir.exists(), "Out dir exists");
    assert!(out_file.exists(), "Out file exists");
    assert!(out_file.is_symlink(), "Out file is a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );
}

#[test]
fn directory_with_file_is_removed() {
    let package = assert_fs::TempDir::new().unwrap();

    let in_dir = package.child("sub_dir");
    let in_file = in_dir.child("file.txt");

    in_file.touch().unwrap();

    let output = assert_fs::TempDir::new().unwrap();
    let out_dir = output.child("sub_dir");
    let out_file = out_dir.child("file.txt");

    out_dir.create_dir_all().unwrap();
    out_file.symlink_to_file(in_file.path()).unwrap();

    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args([
            "--target",
            output.to_str().unwrap(),
            "unlink",
            package.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(in_file.exists(), "In file exists");
    assert!(!out_dir.exists(), "Out dir was removed");
}
