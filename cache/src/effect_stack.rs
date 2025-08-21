#![allow(static_mut_refs)]

use std::rc::Weak;

use once_cell::unsync::Lazy;

use crate::IEffect;

pub(crate) struct EffectStackEntry {
    pub(crate) effect: Weak<dyn IEffect>,
    pub(crate) collecting: bool,
}

static mut EFFECT_STACK: Lazy<Vec<EffectStackEntry>> = Lazy::new(Vec::new);

pub(crate) fn effect_push(effect: Weak<dyn IEffect>, collecting: bool) {
    unsafe { EFFECT_STACK.push(EffectStackEntry { effect, collecting }) }
}

pub(crate) fn effect_peak() -> Option<&'static EffectStackEntry> {
    unsafe { EFFECT_STACK.last() }
}

pub(crate) fn effect_pop(effect: Weak<dyn IEffect>, collecting: bool) {
    let e = unsafe { EFFECT_STACK.pop() }
        .expect("`effect_push` and `effect_pop` are called in pairs and should not be empty.");

    assert!(
        Weak::ptr_eq(&e.effect, &effect),
        "`effect_push` and `effect_pop` are called in pairs and should be the same effect."
    );
    assert_eq!(
        e.collecting, collecting,
        "`effect_push` and `effect_pop` are called in pairs and should have the same collecting flag."
    );
}
