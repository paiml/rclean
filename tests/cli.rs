//test duplicates
use assert_cmd::Command;
use predicates::prelude::*;

const PRG: &str = "rclean";
const DUPE1: &str = "tests/inputs/one.txt";
const DUPE2: &str = "tests/inputs/same-one.txt";
const NOTDUPE: &str = "tests/inputs/three.txt";

#[test]
fn usage() {
    #[allow(clippy::unwrap_used)]
    let mut cmd = Command::cargo_bin(PRG).unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A disk cleanup tool that finds duplicates and storage outliers"));
}

#[test]
fn search() {
    #[allow(clippy::unwrap_used)]
    let mut cmd = Command::cargo_bin(PRG).unwrap();
    cmd.arg("search")
        .arg("tests/inputs")
        .arg("--pattern")
        .arg(".txt")
        .assert()
        .success()
        .stdout(predicate::str::contains(DUPE1))
        .stdout(predicate::str::contains(DUPE2))
        .stdout(predicate::str::contains(NOTDUPE));
}
