#![no_std]
#![no_main]
// Tests for the testing apparatus!

use rps2_libtest::test;

#[test]
fn bruh1() {}

#[test]
fn bruh2() {}

#[test]
#[should_panic]
fn bruh() {
    // panic!("not bruh")
}

#[test]
fn bruh3() {}

#[test]
fn bruh4() {}

#[test]
fn bruh5() {
    panic!("not bruh")
}

#[test]
fn bruh6() {}
