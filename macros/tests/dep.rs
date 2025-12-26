#![allow(static_mut_refs)]

use std::cell::Cell;

use reactive_cache::prelude::*;
use reactive_macros::{memo, signal};

thread_local! {
    static SOURCE_A_CALLED: Cell<bool> = const { Cell::new(false) };
    static SOURCE_B_CALLED: Cell<bool> = const { Cell::new(false) };
    static DERIVED_C_CALLED: Cell<bool> = const { Cell::new(false) };
    static DERIVED_D_CALLED: Cell<bool> = const { Cell::new(false) };
    static DERIVED_E_CALLED: Cell<bool> = const { Cell::new(false) };
}

signal!(
    static mut A: i32 = 10;
);

signal!(
    static mut B: String = 5.to_string();
);

pub fn source_a() -> i32 {
    SOURCE_A_CALLED.set(true);

    *A().get()
}

pub fn source_b() -> i32 {
    SOURCE_B_CALLED.set(true);

    B().get().parse::<i32>().unwrap()
}

#[memo]
pub fn derived_c() -> i32 {
    assert!(!DERIVED_C_CALLED.get());
    DERIVED_C_CALLED.set(true);

    source_a() + source_b()
}

#[memo]
pub fn derived_d() -> i32 {
    assert!(!DERIVED_D_CALLED.get());
    DERIVED_D_CALLED.set(true);

    derived_c() * 2
}

#[memo]
pub fn derived_e() -> i32 {
    assert!(!DERIVED_E_CALLED.get());
    DERIVED_E_CALLED.set(true);

    source_b() - 3
}

// source_a   source_b
//    \         /  \
//     derived_c    derived_e
//         |
//     derived_d

#[test]
fn complex_dependency_memo_test() {
    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    let e1 = derived_e();
    assert!(!SOURCE_A_CALLED.get());
    assert!(SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(DERIVED_E_CALLED.get());
    let d1 = derived_d();
    assert!(SOURCE_A_CALLED.get());
    assert!(SOURCE_B_CALLED.get());
    assert!(DERIVED_C_CALLED.get());
    assert!(DERIVED_D_CALLED.get());
    assert!(DERIVED_E_CALLED.get());
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    let e2 = derived_e();
    let d2 = derived_d();
    let c2 = derived_c();

    assert!(!SOURCE_A_CALLED.get());
    assert!(!SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);

    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);
}

#[test]
fn signal_set_unchanged_test() {
    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    let e1 = derived_e();
    let d1 = derived_d();
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    A().set(10);

    assert!(!SOURCE_A_CALLED.get());
    assert!(!SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());

    let e2 = derived_e();
    let d2 = derived_d();
    let c2 = derived_c();

    assert!(!SOURCE_A_CALLED.get());
    assert!(!SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);
}

#[test]
fn signal_set_value_test() {
    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    let e1 = derived_e();
    let d1 = derived_d();
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    SOURCE_A_CALLED.set(false);
    SOURCE_B_CALLED.set(false);
    DERIVED_C_CALLED.set(false);
    DERIVED_D_CALLED.set(false);
    DERIVED_E_CALLED.set(false);

    A().set(20);
    A().set(10);

    assert!(!SOURCE_A_CALLED.get());
    assert!(!SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());

    let e2 = derived_e();
    assert!(!SOURCE_A_CALLED.get());
    assert!(!SOURCE_B_CALLED.get());
    assert!(!DERIVED_C_CALLED.get());
    assert!(!DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());
    let d2 = derived_d();
    assert!(SOURCE_A_CALLED.get());
    assert!(SOURCE_B_CALLED.get());
    assert!(DERIVED_C_CALLED.get());
    assert!(DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());
    let c2 = derived_c();
    assert!(SOURCE_A_CALLED.get());
    assert!(SOURCE_B_CALLED.get());
    assert!(DERIVED_C_CALLED.get());
    assert!(DERIVED_D_CALLED.get());
    assert!(!DERIVED_E_CALLED.get());

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);
}
