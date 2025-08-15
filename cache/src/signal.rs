use std::rc::Rc;

use crate::{Effect, Observable, call_stack};

pub struct Signal<T, F>
where
    T: Eq + Clone,
    F: Fn() -> T,
{
    value: T,
    f: F,
    dependents: Vec<&'static dyn Observable>,
    effects: Vec<Rc<Effect>>,
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
            effects: vec![],
        }
    }

    fn effects_invoke(&self) {
        self.effects.iter().for_each(|effect| effect.run());
    }

    pub fn get(&'static mut self) -> T {
        if let Some(last) = call_stack::last()
            && !self.dependents.iter().any(|d| std::ptr::eq(*d, *last))
        {
            self.dependents.push(*last);
        }
        if let Some(effect) = call_stack::current_effect_peak()
            && !self.effects.iter().any(|e| Rc::ptr_eq(e, &effect))
        {
            self.effects.push(effect);
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
        self.effects_invoke();

        true
    }

    pub fn update(&'static mut self) -> bool {
        self.set((self.f)())
    }
}
