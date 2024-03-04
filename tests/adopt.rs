use assert_cmd::Command;
use assert_fs::prelude::*;
use std::fs;

#[test]
fn adopt_link_target_not_link() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");
    let other_file = output.child("other.txt");

    in_file.touch().unwrap();
    other_file.write_str("test").unwrap();
    out_file.symlink_to_file(other_file.path()).unwrap();

    let stdout = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .output()
        .unwrap()
        .stdout;

    println!("{}", std::str::from_utf8(&stdout).unwrap());

    let contents = fs::read_to_string(in_file.path()).unwrap();
    let other_contents = fs::read_to_string(other_file.path()).unwrap();

    // Check in file has adopted the other file
    assert!(in_file.exists(), "In file exists");
    assert_eq!(in_file.is_symlink(), false, "In file is not a symlink");
    assert_eq!("test", contents, "In file has now got out file's contents");

    // Check out file has been updated to point to in file
    assert_eq!(out_file.exists(), true, "Out file exists");
    assert_eq!(out_file.is_symlink(), true, "Out file is a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );

    // Check we didn't delete other file
    assert_eq!(other_file.exists(), true, "Other file exists");
    assert_eq!(
        other_file.is_symlink(),
        false,
        "Other file is not a symlink"
    );
    assert_eq!(
        "test", other_contents,
        "Other file has still got original contents"
    );

    package.close().unwrap();
    output.close().unwrap();
}

/// Calling [`std::fs::copy`] when src and dest are the same file seems to truncate the output.
/// Test to make sure we avoid this case.
#[test]
fn dont_copy_to_from_same_file() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("file.txt");
    let out_file = output.child("file.txt");

    in_file.write_str("test").unwrap();

    // Make a different link to in file that isn't detected as a link we could have generated
    // TODO: should we be detecting any links which resolve to our source as our links anyway?
    let mut path = std::path::PathBuf::from("../");
    path.push(package.file_name().unwrap());
    path.push(in_file.file_name().unwrap());

    out_file.symlink_to_file(path).unwrap();

    let stdout = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&[
            "--verbose",
            "--target",
            output.to_str().unwrap(),
            "link",
            "--adopt",
            package.to_str().unwrap(),
        ])
        .output()
        .unwrap()
        .stdout;

    println!("{}", std::str::from_utf8(&stdout).unwrap());

    let contents = fs::read_to_string(in_file.path()).unwrap();

    // Check in file still exists
    assert!(in_file.exists(), "In file exists");
    assert_eq!(in_file.is_symlink(), false, "In file is not a symlink");
    assert_eq!("test", contents, "In file has still got its contents");

    // Check out file still points to in file
    assert_eq!(out_file.exists(), true, "Out file exists");
    assert_eq!(out_file.is_symlink(), true, "Out file is a symlink");
    assert_eq!(
        out_file.read_link().unwrap(),
        in_file.path(),
        "Out file points to in file"
    );

    package.close().unwrap();
    output.close().unwrap();
}
