#![allow(static_mut_refs)]

use cache::Effect;
use macros::{effect, effect_init, memo, signal};

static mut SOURCE_A_CALLED: i32 = 0;
static mut SOURCE_B_CALLED: i32 = 0;
static mut DERIVED_C_CALLED: i32 = 0;
static mut DERIVED_D_CALLED: i32 = 0;
static mut EFFECT_E_CALLED: i32 = 0;
static mut EFFECT_F_CALLED: i32 = 0;

static mut A: i32 = 10;
static mut B: i32 = 5;

#[signal]
pub fn source_a() -> i32 {
    unsafe { SOURCE_A_CALLED += 1 };

    unsafe { A }
}

#[signal]
pub fn source_b() -> i32 {
    unsafe { SOURCE_B_CALLED += 1 };

    unsafe { B }
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

#[effect]
pub fn effect_e() {
    unsafe { EFFECT_E_CALLED += 1 };

    derived_c();
}

// #[effect]
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
    unsafe { A = 0 };
    unsafe { SOURCE_A.update() };
    unsafe { effect_init!(effect_e) };

    unsafe { SOURCE_A_CALLED = 0 };
    unsafe { SOURCE_B_CALLED = 0 };
    unsafe { DERIVED_C_CALLED = 0 };
    unsafe { DERIVED_D_CALLED = 0 };
    unsafe { EFFECT_E_CALLED = 0 };
    unsafe { EFFECT_F_CALLED = 0 };

    unsafe { A = 10 };
    unsafe { SOURCE_A.update() };

    assert_eq!(unsafe { SOURCE_A_CALLED }, 2);
    assert_eq!(unsafe { SOURCE_B_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_C_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_D_CALLED }, 0);
    assert_eq!(unsafe { EFFECT_E_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_F_CALLED }, 0);

    unsafe { A = 0 };
    unsafe { SOURCE_A.update() };
    // unsafe { effect_init!(effect_f) };
    Effect::new(effect_f);

    unsafe { SOURCE_A_CALLED = 0 };
    unsafe { SOURCE_B_CALLED = 0 };
    unsafe { DERIVED_C_CALLED = 0 };
    unsafe { DERIVED_D_CALLED = 0 };
    unsafe { EFFECT_E_CALLED = 0 };
    unsafe { EFFECT_F_CALLED = 0 };

    unsafe { A = 10 };
    unsafe { SOURCE_A.update() };

    assert_eq!(unsafe { SOURCE_A_CALLED }, 2);
    assert_eq!(unsafe { SOURCE_B_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_C_CALLED }, 1);
    assert_eq!(unsafe { DERIVED_D_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_E_CALLED }, 1);
    assert_eq!(unsafe { EFFECT_F_CALLED }, 1);
}
