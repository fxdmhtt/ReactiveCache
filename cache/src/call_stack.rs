#![allow(static_mut_refs)]

use std::rc::Rc;

use once_cell::sync::Lazy;

use crate::{Effect, Observable};

static mut CALL_STACK: Lazy<Vec<&'static dyn Observable>> = Lazy::new(Vec::new);

static mut CURRENT_EFFECT: Option<Rc<Effect>> = None;

pub(crate) fn push(op: &'static dyn Observable) {
    unsafe { CALL_STACK.push(op) }
}

pub(crate) fn last() -> Option<&'static &'static dyn Observable> {
    unsafe { CALL_STACK.last() }
}

pub(crate) fn pop() -> Option<&'static dyn Observable> {
    unsafe { CALL_STACK.pop() }
}

pub(crate) fn current_effect_push(effect: Rc<Effect>) {
    assert!(unsafe { CURRENT_EFFECT.is_none() });
    unsafe { CURRENT_EFFECT = Some(effect) }
}

pub(crate) fn current_effect_peak() -> Option<Rc<Effect>> {
    unsafe { CURRENT_EFFECT.clone() }
}

pub(crate) fn current_effect_pop() -> Rc<Effect> {
    assert!(unsafe { CURRENT_EFFECT.is_some() });
    unsafe { CURRENT_EFFECT.take().unwrap() }
}
