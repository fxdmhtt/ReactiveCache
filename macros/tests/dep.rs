use cache::Observable as _;
use macros::memo;

static mut SOURCE_A_CALLED: bool = false;
static mut SOURCE_B_CALLED: bool = false;
static mut SOURCE_C_CALLED: bool = false;
static mut SOURCE_D_CALLED: bool = false;
static mut SOURCE_E_CALLED: bool = false;

#[memo]
pub fn source_a() -> i32 {
    assert!(!unsafe { SOURCE_A_CALLED });
    unsafe { SOURCE_A_CALLED = true };

    10
}

#[memo]
pub fn source_b() -> i32 {
    assert!(!unsafe { SOURCE_B_CALLED });
    unsafe { SOURCE_B_CALLED = true };

    5
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
    assert!(!unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(!unsafe { SOURCE_C_CALLED });
    assert!(!unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });

    let e1 = derived_e();
    let d1 = derived_d();
    let c1 = derived_c();

    assert_eq!(c1, 15); // 10 + 5
    assert_eq!(d1, 30); // 15 * 2
    assert_eq!(e1, 2); // 5 - 3

    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(unsafe { SOURCE_E_CALLED });

    let e2 = derived_e();
    let d2 = derived_d();
    let c2 = derived_c();

    assert_eq!(c2, c1);
    assert_eq!(d2, d1);
    assert_eq!(e2, e1);

    assert!(unsafe { SOURCE_A_CALLED });
    assert!(unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(unsafe { SOURCE_E_CALLED });

    unsafe { (*SOURCE_A).invalidate() };

    unsafe { SOURCE_A_CALLED = false };
    unsafe { SOURCE_B_CALLED = false };
    unsafe { SOURCE_C_CALLED = false };
    unsafe { SOURCE_D_CALLED = false };
    unsafe { SOURCE_E_CALLED = false };

    let e3 = derived_e();
    let d3 = derived_d();
    let c3 = derived_c();

    assert_eq!(c3, 15);
    assert_eq!(d3, 30);
    assert_eq!(e3, e2);

    assert!(unsafe { SOURCE_A_CALLED });
    assert!(!unsafe { SOURCE_B_CALLED });
    assert!(unsafe { SOURCE_C_CALLED });
    assert!(unsafe { SOURCE_D_CALLED });
    assert!(!unsafe { SOURCE_E_CALLED });
}
