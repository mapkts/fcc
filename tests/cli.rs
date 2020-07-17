use std::fs::OpenOptions;
use std::io::{Read, Write};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{tempdir, NamedTempFile};

macro_rules! testing_files {
    () => {{
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let mut file3 = NamedTempFile::new().unwrap();
        write!(file1, "f1 l1\nf1 l2\nf1 l3").unwrap();
        write!(file2, "f2 l1\nf2 l2\nf2 l3").unwrap();
        write!(file3, "f3 l1\nf3 l2\nf3 l3").unwrap();
        vec![file1, file2, file3]
    }};
}

#[test]
fn bin_fcc_exists() {
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.assert().success();
}

#[test]
fn can_interact_with_stdio() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(files[0].path().to_str().unwrap())
        .assert()
        .stdout(predicate::eq(b"f1 l1\nf1 l2\nf1 l3" as &[u8]));
}

#[test]
fn can_accept_paths_through_arg_i() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.arg("-i")
        .arg(files[0].path().to_str().unwrap())
        .arg(files[1].path().to_str().unwrap())
        .arg(files[2].path().to_str().unwrap())
        .assert()
        .stdout(predicate::eq(
            b"f1 l1\nf1 l2\nf1 l3f2 l1\nf2 l2\nf2 l3f3 l1\nf3 l2\nf3 l3" as &[u8],
        ));
}

#[test]
fn can_accept_space_separated_paths_through_stdin() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{} {} {}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3f2 l1\nf2 l2\nf2 l3f3 l1\nf3 l2\nf3 l3" as &[u8],
    ));
}

#[test]
fn can_accept_newline_separated_paths_through_stdin() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3f2 l1\nf2 l2\nf2 l3f3 l1\nf3 l2\nf3 l3" as &[u8],
    ));
}

#[test]
fn can_write_output_to_a_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("output.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .append(true)
        .open(file_path.clone())
        .unwrap();
    let input_files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        input_files[0].path().to_str().unwrap(),
        input_files[1].path().to_str().unwrap(),
        input_files[2].path().to_str().unwrap()
    ))
    .arg("-o")
    .arg(file_path.to_str().unwrap())
    .unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    assert_eq!(
        buf.as_slice(),
        b"f1 l1\nf1 l2\nf1 l3f2 l1\nf2 l2\nf2 l3f3 l1\nf3 l2\nf3 l3" as &[u8]
    );
}

#[test]
fn arg_newline_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-n")
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3\nf2 l1\nf2 l2\nf2 l3\nf3 l1\nf3 l2\nf3 l3\n" as &[u8],
    ));
}

#[test]
fn arg_header_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-H")
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3f2 l2\nf2 l3f3 l2\nf3 l3" as &[u8],
    ));
}

#[test]
fn arg_crlf_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-n")
    .arg("--crlf")
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3\r\nf2 l1\nf2 l2\nf2 l3\r\nf3 l1\nf3 l2\nf3 l3\r\n" as &[u8],
    ));
}

#[test]
fn arg_padding_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg("(padding)")
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf1 l3(padding)f2 l1\nf2 l2\nf2 l3(padding)f3 l1\nf3 l2\nf3 l3(padding)"
            as &[u8],
    ));
}

#[test]
fn arg_skip_start_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("1")
    .assert()
    .stdout(predicate::eq(
        b"f1 l2\nf1 l3f2 l2\nf2 l3f3 l2\nf3 l3" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("2")
    .assert()
    .stdout(predicate::eq(b"f1 l3f2 l3f3 l3" as &[u8]));
}

#[test]
fn arg_skip_end_works() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("1")
    .assert()
    .stdout(predicate::eq(
        b"f1 l1\nf1 l2\nf2 l1\nf2 l2\nf3 l1\nf3 l2\n" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("2")
    .assert()
    .stdout(predicate::eq(b"f1 l1\nf2 l1\nf3 l1\n" as &[u8]));
}
