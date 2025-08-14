use crate::{Observable, call_stack};

pub struct Signal<T, F>
where
    T: Eq + Clone,
    F: Fn() -> T,
{
    value: T,
    f: F,
    dependents: Vec<&'static dyn Observable>,
}

impl<T, F> Observable for Signal<T, F>
where
    T: Eq + Clone,
    F: Fn() -> T,
{
    fn invalidate(&'static self) {
        self.dependents
            .iter()
            .for_each(|dependent| dependent.invalidate());
    }
}

impl<T, F> Signal<T, F>
where
    T: Eq + Clone,
    F: Fn() -> T,
{
    pub fn new(f: F) -> Self {
        let value = f();
        Signal {
            value,
            f,
            dependents: vec![],
        }
    }

    pub fn get(&'static mut self) -> T {
        if let Some(last) = call_stack::last() {
            self.dependents.push(*last);
        }

        let result: T = (self.f)();
        self.set(result.clone());

        result
    }

    pub fn set(&'static mut self, value: T) -> bool {
        if self.value == value {
            return false;
        }

        self.value = value;

        self.invalidate();

        true
    }
}
