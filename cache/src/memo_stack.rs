#![allow(static_mut_refs)]

use std::rc::Weak;

use once_cell::sync::Lazy;

use crate::IMemo;

static mut MEMO_STACK: Lazy<Vec<Weak<dyn IMemo>>> = Lazy::new(Vec::new);

pub(crate) fn push(op: Weak<dyn IMemo>) {
    unsafe { MEMO_STACK.push(op) }
}

pub(crate) fn last() -> Option<&'static Weak<dyn IMemo>> {
    unsafe { MEMO_STACK.last() }
}

pub(crate) fn pop() -> Option<Weak<dyn IMemo>> {
    unsafe { MEMO_STACK.pop() }
}
