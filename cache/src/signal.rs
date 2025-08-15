use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use mvvm::System::ComponentModel::ObservableProperty;

use crate::{Effect, Observable, call_stack};

pub struct Signal<T>
where
    T: Eq + Default + 'static,
{
    prop: ObservableProperty<'static, T>,
    dependents: RefCell<Vec<&'static dyn Observable>>,
    effects: RefCell<Vec<Rc<Effect>>>,
}

impl<T> Signal<T>
where
    T: Eq + Default + 'static,
{
    fn invalidate(&self) {
        self.dependents
            .borrow()
            .iter()
            .for_each(|dependent| dependent.invalidate());
    }

    fn flush_effects(&self) {
        self.effects.borrow().iter().for_each(|effect| effect.run());
    }

    pub fn new(value: Option<T>) -> Rc<Self> {
        let s = Rc::new(Signal {
            prop: value.map_or_else(Default::default, ObservableProperty::new),
            dependents: vec![].into(),
            effects: vec![].into(),
        });

        let weak = Rc::downgrade(&s);
        s.prop.PropertyChanging.borrow_mut().add(move |_| {
            if let Some(s) = weak.upgrade() {
                s.invalidate();
            }
        });

        let weak = Rc::downgrade(&s);
        s.prop.PropertyChanged.borrow_mut().add(move |_| {
            if let Some(s) = weak.upgrade() {
                s.flush_effects();
            }
        });

        s
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
        if let Some(effect) = call_stack::current_effect_peak()
            && !self.effects.borrow().iter().any(|e| Rc::ptr_eq(e, &effect))
        {
            self.effects.borrow_mut().push(effect);
        }

        self.prop.GetValue()
    }

    pub fn set(&self, value: T) -> bool {
        self.prop.SetValue(value)
    }
}
