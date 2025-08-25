use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::{Effect, IMemo, IObservable, effect_stack::EffectStackEntry};

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
/// - `T`: The type of the value stored in the signal. Must implement `Eq`.
///
/// # Examples
///
/// ## Basic usage
/// ```
/// use std::rc::Rc;
/// use reactive_cache::Signal;
///
/// let signal = Signal::new(10);
/// assert_eq!(*signal.get(), 10);
///
/// signal.set(20);
/// assert_eq!(*signal.get(), 20);
/// ```
///
/// ## Using inside a struct
/// ```
/// use std::rc::Rc;
/// use reactive_cache::Signal;
///
/// struct ViewModel {
///     counter: Rc<Signal<i32>>,
///     name: Rc<Signal<String>>,
/// }
///
/// let vm = ViewModel {
///     counter: Signal::new(0).into(),
///     name: Signal::new("Alice".to_string()).into(),
/// };
///
/// assert_eq!(*vm.counter.get(), 0);
/// assert_eq!(*vm.name.get(), "Alice");
///
/// vm.counter.set(1);
/// vm.name.set("Bob".into());
///
/// assert_eq!(*vm.counter.get(), 1);
/// assert_eq!(*vm.name.get(), "Bob");
/// ```
pub struct Signal<T> {
    value: RefCell<T>,
    dependents: RefCell<Vec<Weak<dyn IMemo>>>,
    effects: RefCell<Vec<Weak<Effect>>>,
}

impl<T> Signal<T> {
    /// Re-runs all dependent effects that are still alive.
    ///
    /// This is triggered after the signal's value has changed.  
    /// Dead effects (already dropped) are cleaned up automatically.
    fn flush_effects(&self) {
        // When triggering an Effect, dependencies are not collected for that Effect.
        self.effects.borrow_mut().retain(|w| {
            if let Some(e) = w.upgrade() {
                crate::effect::run_untracked(&e);
                true
            } else {
                false
            }
        });
    }

    /// Called after the value is updated.  
    /// Triggers all dependent effects.
    #[allow(non_snake_case)]
    fn OnPropertyChanged(&self) {
        self.flush_effects()
    }

    /// Called before the value is updated.  
    /// Invalidates all memoized computations depending on this signal.
    #[allow(non_snake_case)]
    fn OnPropertyChanging(&self) {
        self.invalidate()
    }

    /// Creates a new `Signal` with the given initial value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use std::rc::Rc;
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(10);
    /// assert_eq!(*signal.get(), 10);
    /// ```
    ///
    /// Using inside a struct:
    /// ```
    /// use std::rc::Rc;
    /// use reactive_cache::Signal;
    ///
    /// struct ViewModel {
    ///     counter: Rc<Signal<i32>>,
    ///     name: Rc<Signal<String>>,
    /// }
    ///
    /// let vm = ViewModel {
    ///     counter: Signal::new(0),
    ///     name: Signal::new("Alice".to_string()),
    /// };
    ///
    /// assert_eq!(*vm.counter.get(), 0);
    /// assert_eq!(*vm.name.get(), "Alice");
    ///
    /// // Update values
    /// assert!(vm.counter.set(1));
    /// assert!(vm.name.set("Bob".into()));
    ///
    /// assert_eq!(*vm.counter.get(), 1);
    /// assert_eq!(*vm.name.get(), "Bob");
    /// ```
    pub fn new(value: T) -> Rc<Self>
    {
        Signal {
            value: value.into(),
            dependents: vec![].into(),
            effects: vec![].into(),
        }
        .into()
    }

    /// Gets a reference to the current value, tracking dependencies
    /// and effects if inside a reactive context.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(42);
    /// assert_eq!(*signal.get(), 42);
    /// ```
    pub fn get(&self) -> Ref<'_, T> {
        self.dependency_collection();

        // Track effects in the call stack
        if let Some(EffectStackEntry {
            effect: e,
            collecting,
        }) = crate::effect_stack::effect_peak()
            && *collecting
            && !self.effects.borrow().iter().any(|w| Weak::ptr_eq(w, e))
        {
            self.effects.borrow_mut().push(e.clone());
        }

        self.value.borrow()
    }

    /// Sets the value of the signal.
    ///
    /// Returns `true` if the value changed, all dependent memos are
    /// invalidated and dependent effects were triggered.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Signal;
    ///
    /// let signal = Signal::new(5);
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

impl<T> IObservable for Signal<T> {
    fn dependents(&self) -> &RefCell<Vec<Weak<dyn IMemo>>> {
        &self.dependents
    }
}
