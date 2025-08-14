#![allow(static_mut_refs)]

use once_cell::sync::Lazy;

use crate::Observable;

static mut CALL_STACK: Lazy<Vec<&'static dyn Observable>> = Lazy::new(Vec::new);

pub fn push(op: &'static dyn Observable) {
    unsafe { CALL_STACK.push(op) }
}

pub fn last() -> Option<&'static &'static dyn Observable> {
    unsafe { CALL_STACK.last() }
}

pub fn pop() -> Option<&'static dyn Observable> {
    unsafe { CALL_STACK.pop() }
}
