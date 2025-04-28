use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn handle_missing_link() {
    let package = assert_fs::TempDir::new().unwrap();
    let output = assert_fs::TempDir::new().unwrap();
    let in_file = package.child("a.txt");
    let out_file = output.child("a.txt");

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
    assert!(out_file.exists(), "Symlink target doesn't exist");
    assert_eq!(out_file.read_link().unwrap(), in_file.path());

    std::fs::remove_file(&out_file).unwrap();

    assert!(!out_file.exists(), "Symlink target exists");
    assert!(!out_file.is_symlink(), "Out file is still a link");

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

    package.close().unwrap();
    output.close().unwrap();
}
