use crate::{Observable, call_stack, remove_from_cache, store_in_cache, touch};

pub struct Memo<T, F>
where
    T: Clone,
    F: Fn() -> T,
{
    f: F,
    dependents: Vec<&'static dyn Observable>,
}

impl<T, F> Observable for Memo<T, F>
where
    T: Clone,
    F: Fn() -> T,
{
    fn invalidate(&'static self) {
        remove_from_cache(self);
        self.dependents
            .iter()
            .for_each(|dependent| dependent.invalidate());
    }
}

impl<T, F> Memo<T, F>
where
    T: Clone,
    F: Fn() -> T,
{
    pub fn new(f: F) -> Self {
        Memo {
            f,
            dependents: vec![],
        }
    }

    pub fn get(&'static mut self) -> T {
        if let Some(last) = call_stack::last()
            && !self.dependents.iter().any(|d| std::ptr::eq(*d, *last))
        {
            self.dependents.push(*last);
        }

        call_stack::push(self);

        let rc = if let Some(rc) = touch(self) {
            rc
        } else {
            let result: T = (self.f)();
            store_in_cache(self, result)
        };

        call_stack::pop();

        (*rc).clone()
    }
}
