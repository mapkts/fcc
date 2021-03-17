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
        write!(file1, "111 112\n121 122\n131 132\n").unwrap();
        write!(file2, "211 212\n221 222\n231 232\n").unwrap();
        write!(file3, "311 312\n332 322\n331 332").unwrap();
        vec![file1, file2, file3]
    }};
}

#[test]
fn bin_fcc_exists() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(files[0].path().to_str().unwrap())
        .assert()
        .stdout(predicate::eq(b"111 112\n121 122\n131 132\n" as &[u8]));
}

#[test]
fn can_interact_with_stdio() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(files[0].path().to_str().unwrap())
        .assert()
        .stdout(predicate::eq(b"111 112\n121 122\n131 132\n" as &[u8]));
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
            b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
                as &[u8],
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
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
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
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));
}

#[test]
fn can_accept_newline_and_space_separated_paths_through_stdin() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{} {}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
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
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8]
    );
}

#[test]
fn arg_skip_head_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("0")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

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
        b"121 122\n131 132\n221 222\n231 232\n332 322\n331 332" as &[u8],
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
    .stdout(predicate::eq(b"131 132\n231 232\n331 332" as &[u8]));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("3")
    .assert()
    .stdout(predicate::eq(b"" as &[u8]));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("4")
    .assert()
    .failure();
}

#[test]
fn arg_skip_tail_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("0")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

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
        b"111 112\n121 122\n211 212\n221 222\n311 312\n332 322\n" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("3")
    .assert()
    .stdout(predicate::eq(b"" as &[u8]));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("4")
    .assert()
    .failure();
}

#[test]
fn arg_skip_head_once_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-S")
    .arg("0")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-S")
    .arg("1")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n221 222\n231 232\n332 322\n331 332" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-S")
    .arg("2")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n231 232\n331 332" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-S")
    .arg("3")
    .assert()
    .stdout(predicate::eq(b"111 112\n121 122\n131 132\n" as &[u8]));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("4")
    .assert()
    .failure();
}

#[test]
fn arg_skip_tail_onces_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-E")
    .arg("0")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-E")
    .arg("1")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n211 212\n221 222\n311 312\n332 322\n331 332" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-E")
    .arg("3")
    .assert()
    .stdout(predicate::eq(b"311 312\n332 322\n331 332" as &[u8]));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("4")
    .assert()
    .failure();
}

#[test]
fn arg_skip_head_bytes_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("0")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("4")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"112\n121 122\n131 132\n212\n221 222\n231 232\n312\n332 322\n331 332" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("8")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"121 122\n131 132\n221 222\n231 232\n332 322\n331 332" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("24")
    .arg("-m")
    .arg("bytes")
    .assert()
    .failure();
}

#[test]
fn arg_skip_tail_bytes_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("0")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("4")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 211 212\n221 222\n231 311 312\n332 322\n331" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("8")
    .arg("-m")
    .arg("bytes")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n211 212\n221 222\n311 312\n332 322" as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("24")
    .arg("-m")
    .arg("bytes")
    .assert()
    .failure();
}

#[test]
fn arg_headonce_works_as_expected() {
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
        b"111 112\n121 122\n131 132\n221 222\n231 232\n332 322\n331 332" as &[u8],
    ));
}

#[test]
fn arg_tailonce_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-T")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n211 212\n221 222\n311 312\n332 322\n331 332" as &[u8],
    ));
}

#[test]
fn arg_padding_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg(" padding ")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n padding 211 212\n221 222\n231 232\n padding 311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg(" padding ")
    .arg("--pad-mode")
    .arg("between")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n padding 211 212\n221 222\n231 232\n padding 311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg(" padding ")
    .arg("--pad-mode")
    .arg("beforestart")
    .assert()
    .stdout(predicate::eq(
        b" padding 111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg(" padding ")
    .arg("--pad-mode")
    .arg("afterend")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332 padding "
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-p")
    .arg(" padding ")
    .arg("--pad-mode")
    .arg("all")
    .assert()
    .stdout(predicate::eq(
        b" padding 111 112\n121 122\n131 132\n padding 211 212\n221 222\n231 232\n padding 311 312\n332 322\n331 332 padding "
            as &[u8],
    ));
}

#[test]
fn arg_newline_works_as_expected() {
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
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332\n"
            as &[u8],
    ));
}

#[test]
fn arg_newline_style_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-n")
    .arg("-N")
    .arg("lf")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332\n"
            as &[u8],
    ));

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-n")
    .arg("-N")
    .arg("crlf")
    .assert()
    .stdout(predicate::eq(
        b"111 112\n121 122\n131 132\n211 212\n221 222\n231 232\n311 312\n332 322\n331 332\r\n"
            as &[u8],
    ));
}

#[test]
fn args_mixed_works_as_expected() {
    let files = testing_files!();
    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-n")
    .arg("-s")
    .arg("1")
    .arg("-E")
    .arg("1")
    .arg("-p")
    .arg(" padding ")
    .arg("-P")
    .arg("all")
    .assert()
    .stdout(predicate::eq(
        b" padding 121 122\n padding 221 222\n padding 332 322\n331 332\n padding " as &[u8],
    ));
}

#[test]
fn skip_heads_cannot_provide_at_the_same_time() {
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
    .arg("-S")
    .arg("1")
    .assert()
    .failure();

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-s")
    .arg("1")
    .arg("-H")
    .arg("1")
    .assert()
    .failure();

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-S")
    .arg("1")
    .arg("-H")
    .arg("1")
    .assert()
    .failure();
}

#[test]
fn skip_tails_cannot_provide_at_the_same_time() {
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
    .arg("-E")
    .arg("1")
    .assert()
    .failure();

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-e")
    .arg("1")
    .arg("-T")
    .arg("1")
    .assert()
    .failure();

    let mut cmd = Command::cargo_bin("fcc").unwrap();
    cmd.write_stdin(format!(
        "{}\n{}\n{}",
        files[0].path().to_str().unwrap(),
        files[1].path().to_str().unwrap(),
        files[2].path().to_str().unwrap()
    ))
    .arg("-E")
    .arg("1")
    .arg("-T")
    .arg("1")
    .assert()
    .failure();
}
