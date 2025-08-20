use std::{
    cell::{Ref, RefCell},
    rc::Weak,
};

use crate::{IEffect, Observable, call_stack};

/// A reactive signal that holds a value, tracks dependencies, and triggers effects.
///
/// `Signal<T>` behaves similarly to a traditional "Property" (getter/setter),
/// but on top of that, it automatically tracks which reactive computations
/// or effects access it. When its value changes, all dependent effects
/// are automatically re-run.
///
/// In short:
/// - Like a Property: provides `get()` and `set()` for accessing and updating the value.
/// - Adds tracking: automatically records dependencies when read inside reactive contexts,
///   and automatically triggers dependent `Effect`s when updated.
///
/// # Type Parameters
///
/// - `T`: The type of the value stored in the signal. Must implement `Eq + Default`.
pub struct Signal<T> {
    value: RefCell<T>,
    dependents: RefCell<Vec<&'static dyn Observable>>,
    effects: RefCell<Vec<Weak<dyn IEffect>>>,
}

impl<T> Signal<T> {
    /// Invalidates all dependent observables.
    fn invalidate(&self) {
        self.dependents.borrow().iter().for_each(|d| d.invalidate());
    }

    /// Runs all dependent effects.
    fn flush_effects(&self) {
        // When triggering an Effect, dependencies are not collected for that Effect.
        //
        // The issue arises from the creation of Effect A, which may trigger Effect B.
        // In Effect B, the signal get operation causes incorrect tracking of Effect A.
        // In this case, dependency tracking for Effect A should not be performed
        // until Effect B exits the triggering phase.
        //
        // The root cause of this issue is a temporary violation of the assumption that
        // "an Effect is always the end point of the call chain."
        let creating_effect = unsafe { crate::call_stack::CREATING_EFFECT };
        unsafe { crate::call_stack::CREATING_EFFECT = false };

        self.effects.borrow_mut().retain(|w| {
            if let Some(e) = w.upgrade() {
                e.run();
                true
            } else {
                false
            }
        });

        unsafe { crate::call_stack::CREATING_EFFECT = creating_effect };
    }

    #[allow(non_snake_case)]
    fn OnPropertyChanged(&self) {
        self.flush_effects()
    }

    #[allow(non_snake_case)]
    fn OnPropertyChanging(&self) {
        self.invalidate()
    }

    /// Creates a new `Signal` with the given initial value.
    ///
    /// If `None` is provided, `T::default()` is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(Some(10));
    /// assert_eq!(*signal.get(), 10);
    ///
    /// let default_signal: Signal<i32> = Signal::new(None);
    /// assert_eq!(*default_signal.get(), 0);
    /// ```
    pub fn new(value: Option<T>) -> Self
    where
        T: Default,
    {
        Signal {
            value: value.unwrap_or_default().into(),
            dependents: vec![].into(),
            effects: vec![].into(),
        }
    }

    /// Gets a reference to the current value, tracking dependencies and effects if inside a reactive context.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(Some(42));
    /// assert_eq!(*signal.get(), 42);
    /// ```
    pub fn get(&self) -> Ref<'_, T> {
        // Track observables in the call stack
        if let Some(last) = call_stack::last()
            && !self
                .dependents
                .borrow()
                .iter()
                .any(|d| std::ptr::eq(*d, *last))
        {
            self.dependents.borrow_mut().push(*last);
        }

        // Track effects in the call stack
        if unsafe { call_stack::CREATING_EFFECT }
            && let Some(e) = call_stack::current_effect_peak()
            && !self.effects.borrow().iter().any(|w| Weak::ptr_eq(w, &e))
        {
            self.effects.borrow_mut().push(e);
        }

        self.value.borrow()
    }

    /// Sets the value of the signal.
    ///
    /// Returns `true` if the value changed and dependent effects were triggered.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(Some(5));
    /// assert_eq!(signal.set(10), true);
    /// assert_eq!(*signal.get(), 10);
    ///
    /// // Setting to the same value returns false
    /// assert_eq!(signal.set(10), false);
    /// ```
    pub fn set(&self, value: T) -> bool
    where
        T: Eq,
    {
        if *self.value.borrow() == value {
            return false;
        }

        self.OnPropertyChanging();

        *self.value.borrow_mut() = value;

        self.OnPropertyChanged();

        true
    }
}
