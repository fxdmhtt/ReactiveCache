#![allow(static_mut_refs)]

use std::rc::Rc;

use reactive_macros::{effect, memo, signal};

static mut SOURCE_A_CALLED: i32 = 0;
static mut SOURCE_B_CALLED: i32 = 0;
static mut DERIVED_C_CALLED: i32 = 0;
static mut DERIVED_D_CALLED: i32 = 0;
static mut EFFECT_E_CALLED: i32 = 0;
static mut EFFECT_F_CALLED: i32 = 0;

signal!(
    static mut A: i32 = 10;
);

signal!(
    static mut B: i32 = 5;
);

pub fn source_a() -> i32 {
    unsafe { SOURCE_A_CALLED += 1 };

    A_get()
}

pub fn source_b() -> i32 {
    unsafe { SOURCE_B_CALLED += 1 };

    B()
}

#[memo]
pub fn derived_c() -> i32 {
    unsafe { DERIVED_C_CALLED += 1 };

    source_a() + source_b()
}

#[memo]
pub fn derived_d() -> i32 {
    unsafe { DERIVED_D_CALLED += 1 };

    derived_c() * 2
}

pub fn effect_e() {
    unsafe { EFFECT_E_CALLED += 1 };

    derived_c();
}

pub fn effect_f() {
    unsafe { EFFECT_F_CALLED += 1 };

    derived_d();
}

// source_a   source_b
//    \         /
//     derived_c - effect_e
//         |
//     derived_d - effect_f

#[test]
fn simple_effect_test() {
    A_set(0);
    effect!(effect_e);
    effect!(|| { effect_f() });

    unsafe { SOURCE_A_CALLED = 0 };
    unsafe { SOURCE_B_CALLED = 0 };
    unsafe { DERIVED_C_CALLED = 0 };
    unsafe { DERIVED_D_CALLED = 0 };
    unsafe { EFFECT_E_CALLED = 0 };
    unsafe { EFFECT_F_CALLED = 0 };

    A_set(10);

    assert_eq!(unsafe { SOURCE_A_CALLED }, 0);
    assert_eq!(unsafe { SOURCE_B_CALLED }, 0);
    assert_eq!(unsafe { DERIVED_C_CALLED }, 0);
    assert_eq!(unsafe { DERIVED_D_CALLED }, 0);
    assert_eq!(unsafe { EFFECT_E_CALLED }, 0);
    assert_eq!(unsafe { EFFECT_F_CALLED }, 0);

    A_set(0);
    let _ = Rc::into_raw(effect!(effect_e));

    unsafe { SOURCE_A_CALLED = 0 };
    unsafe { SOURCE_B_CALLED = 0 };
    unsafe { DERIVED_C_CALLED = 0 };
    unsafe { DERIVED_D_CALLED = 0 };
    unsafe { EFFECT_E_CALLED = 0 };
    unsafe { EFFECT_F_CALLED = 0 };

    A_set(10);

    assert_eq!(unsafe { SOURCE_A_CALLED }, 1);
    assert_eq!(unsafe { SOURCE_B_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_C_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_D_CALLED }, 0);
    assert_eq!(unsafe { EFFECT_E_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_F_CALLED }, 0);

    A_set(0);
    let _ = Rc::into_raw(effect!(|| { effect_f() }));

    unsafe { SOURCE_A_CALLED = 0 };
    unsafe { SOURCE_B_CALLED = 0 };
    unsafe { DERIVED_C_CALLED = 0 };
    unsafe { DERIVED_D_CALLED = 0 };
    unsafe { EFFECT_E_CALLED = 0 };
    unsafe { EFFECT_F_CALLED = 0 };

    A_set(10);

    assert_eq!(unsafe { SOURCE_A_CALLED }, 1);
    assert_eq!(unsafe { SOURCE_B_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_C_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_D_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_E_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_F_CALLED }, 1);
}
