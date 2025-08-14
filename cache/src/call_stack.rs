use crate::Observable;

static mut CALL_STACK: Option<Vec<&'static dyn Observable>> = None;

fn call_stack() -> &'static mut Vec<&'static dyn Observable> {
    #[allow(static_mut_refs)]
    unsafe {
        CALL_STACK.get_or_insert_with(Vec::new)
    }
}

pub fn push(op: &'static dyn Observable) {
    call_stack().push(op)
}

pub fn last() -> Option<&'static &'static dyn Observable> {
    call_stack().last()
}

pub fn pop() -> Option<&'static dyn Observable> {
    call_stack().pop()
}
