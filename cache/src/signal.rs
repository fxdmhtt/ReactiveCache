use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::{Effect, Observable, call_stack};

pub struct Signal<T>
where
    T: Eq + Default + 'static,
{
    value: RefCell<T>,
    dependents: RefCell<Vec<&'static dyn Observable>>,
    effects: RefCell<Vec<Weak<Effect>>>,
}

impl<T> Signal<T>
where
    T: Eq + Default + 'static,
{
    fn invalidate(&self) {
        self.dependents.borrow().iter().for_each(|d| d.invalidate());
    }

    fn flush_effects(&self) {
        self.effects.borrow_mut().retain(|w| {
            if let Some(e) = w.upgrade() {
                e.run();
                true
            } else {
                false
            }
        });
    }

    #[allow(non_snake_case)]
    fn OnPropertyChanged(&self) {
        self.flush_effects()
    }

    #[allow(non_snake_case)]
    fn OnPropertyChanging(&self) {
        self.invalidate()
    }

    pub fn new(value: Option<T>) -> Self {
        Signal {
            value: value.unwrap_or_default().into(),
            dependents: vec![].into(),
            effects: vec![].into(),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        if let Some(last) = call_stack::last()
            && !self
                .dependents
                .borrow()
                .iter()
                .any(|d| std::ptr::eq(*d, *last))
        {
            self.dependents.borrow_mut().push(*last);
        }
        if let Some(e) = call_stack::current_effect_peak()
            && !self
                .effects
                .borrow()
                .iter()
                .any(|w| w.as_ptr() == Rc::as_ptr(&e))
        {
            self.effects.borrow_mut().push(Rc::downgrade(&e));
        }

        self.value.borrow()
    }

    pub fn set(&self, value: T) -> bool {
        if *self.value.borrow() == value {
            return false;
        }

        self.OnPropertyChanging();

        *self.value.borrow_mut() = value;

        self.OnPropertyChanged();

        true
    }
}
