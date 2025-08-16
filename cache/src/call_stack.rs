#![allow(static_mut_refs)]

use std::rc::Weak;

use once_cell::sync::Lazy;

use crate::{IEffect, Observable};

static mut CALL_STACK: Lazy<Vec<&'static dyn Observable>> = Lazy::new(Vec::new);

static mut CREATING_EFFECT: Option<Weak<dyn IEffect>> = None;

pub(crate) fn push(op: &'static dyn Observable) {
    unsafe { CALL_STACK.push(op) }
}

pub(crate) fn last() -> Option<&'static &'static dyn Observable> {
    unsafe { CALL_STACK.last() }
}

pub(crate) fn pop() -> Option<&'static dyn Observable> {
    unsafe { CALL_STACK.pop() }
}

pub(crate) fn creating_effect_push(effect: Weak<dyn IEffect>) {
    assert!(unsafe { CREATING_EFFECT.is_none() });
    unsafe { CREATING_EFFECT = Some(effect) }
}

pub(crate) fn creating_effect_peak() -> Option<Weak<dyn IEffect>> {
    unsafe { CREATING_EFFECT.clone() }
}

pub(crate) fn creating_effect_pop() -> Weak<dyn IEffect> {
    assert!(unsafe { CREATING_EFFECT.is_some() });
    unsafe { CREATING_EFFECT.take().unwrap() }
}
