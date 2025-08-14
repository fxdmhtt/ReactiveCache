#![allow(static_mut_refs)]

use cache::Observable as _;
use macros::{memo, signal};

static mut SOURCE_A_CALLED: bool = false;
static mut SOURCE_B_CALLED: bool = false;
static mut SOURCE_C_CALLED: bool = false;
static mut SOURCE_D_CALLED: bool = false;
static mut SOURCE_E_CALLED: bool = false;

static mut A: i32 = 10;
static mut B: i32 = 5;

#[signal]
pub fn source_a() -> i32 {
    unsafe { SOURCE_A_CALLED = true };

    unsafe { A }
}

#[signal]
pub fn source_b() -> i32 {
    unsafe { SOURCE_B_CALLED = true };

    unsafe { B }
}

#[memo]
pub fn derived_c() -> i32 {
    assert!(!unsafe { SOURCE_C_CALLED });
    unsafe { SOURCE_C_CALLED = true };

    source_a() + source_b()
}

#[memo]
pub fn derived_d() -> i32 {
    assert!(!unsafe { SOURCE_D_CALLED });
    unsafe { SOURCE_D_CALLED = true };

    derived_c() * 2
}

#[memo]
pub fn derived_e() -> i32 {
    assert!(!unsafe { SOURCE_E_CALLED });
    unsafe { SOURCE_E_CALLED = true };

    source_b() - 3
}

// source_a   source_b
//    \         /  \
//     derived_c    derived_e
//         |
//     derived_d

#[test]
fn complex_dependency_memo_test() {
    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e1 = derived_e();
    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(unsafe { SOURCE_E_CALLED });
    let d1 = derived_d();
    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(unsafe { SOURCE_E_CALLED });
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    unsafe { A = 10 };

    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e2 = derived_e();
    let d2 = derived_d();
    let c2 = derived_c();

    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);

    unsafe { SOURCE_A.invalidate() };

    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e3 = derived_e();
    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });
    let d3 = derived_d();
    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });
    let c3 = derived_c();
    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });

    assert_eq!(c3, 15);
    assert_eq!(d3, 30);
    assert_eq!(e3, e2);
}

#[test]
fn signal_set_unchanged_test() {
    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e1 = derived_e();
    let d1 = derived_d();
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    unsafe { SOURCE_A.set(10) };

    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e2 = derived_e();
    let d2 = derived_d();
    let c2 = derived_c();

    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);
}

#[test]
fn signal_set_value_test() {
    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e1 = derived_e();
    let d1 = derived_d();
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    unsafe { SOURCE_A.set(20) };

    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e2 = derived_e();
    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });
    let d2 = derived_d();
    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });
    let c2 = derived_c();
    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);
}
