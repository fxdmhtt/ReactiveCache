#![allow(static_mut_refs)]

use std::{cell::Cell, rc::Rc};

use reactive_cache::effect;
use reactive_macros::{memo, ref_signal, signal};

static mut SOURCE_A_CALLED: i32 = 0;
static mut SOURCE_B_CALLED: i32 = 0;
static mut DERIVED_C_CALLED: i32 = 0;
static mut DERIVED_D_CALLED: i32 = 0;
static mut EFFECT_E_CALLED: i32 = 0;
static mut EFFECT_F_CALLED: i32 = 0;

signal!(
    static mut A: i32 = 10;
);

ref_signal!(
    static mut B: String = 5.to_string();
);

pub fn source_a() -> i32 {
    unsafe { SOURCE_A_CALLED += 1 };

    A()
}

pub fn source_b() -> i32 {
    unsafe { SOURCE_B_CALLED += 1 };

    B_get().parse::<i32>().unwrap()
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
fn complex_dependency_effect_test() {
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

signal!(
    static mut SWITCH_A: bool = true;
);

signal!(
    static mut SWITCH_B: i32 = 10;
);

#[test]
fn switch_effect_test() {
    let b_rst = Rc::new(Cell::new(0));

    let b_rst_clone = b_rst.clone();
    let _ = Rc::into_raw(effect!(
        move || {
            match SWITCH_A() {
                true => {}
                false => {
                    b_rst_clone.set(*SWITCH_B_get());
                }
            }
        },
        || {
            SWITCH_B();
        }
    ));

    assert_eq!(b_rst.get(), 0);

    SWITCH_A_set(false);
    assert_eq!(b_rst.get(), 10);

    SWITCH_B_set(20);
    assert_eq!(b_rst.get(), 20); // SWITCH_B is reactive because it is included in the deps closure of `effect!`
}
