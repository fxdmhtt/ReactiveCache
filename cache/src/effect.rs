use std::rc::Rc;

/// A reactive effect that runs a closure whenever its dependencies change.
///
/// `Effect<F>` behaves similarly to an "event listener" or a callback,
/// but it is automatically tied to any signals or memos it reads during execution.
/// When those dependencies change, the effect will re-run.
///
/// Note: The closure runs **immediately upon creation** via `Effect::wrap`,
/// so the effect is always initialized with an up-to-date value.
///
/// In short:
/// - Like a callback: wraps a closure of type `F` and runs it.
/// - Adds tracking: automatically re-runs when dependent signals change.
/// - Runs once immediately at creation.
///
/// # Type Parameters
///
/// - `F`: The closure type wrapped by this effect. Must implement `Fn()`.
///   The closure is executed immediately upon creation and tracked for reactive updates.
pub struct Effect<F>
where
    F: Fn(),
{
    f: F,
}

impl<F> Effect<F>
where
    F: Fn(),
{
    fn new_inner<D>(f: F, deps: Option<D>) -> Rc<dyn IEffect>
    where
        F: 'static,
        D: Fn() + 'static,
    {
        let e: Rc<dyn IEffect> = Rc::new(Effect { f });

        unsafe { crate::call_stack::CREATING_EFFECT = true };
        crate::current_effect_push(Rc::downgrade(&e));

        if let Some(deps) = deps {
            deps();
        }
        e.run();

        crate::current_effect_pop();
        unsafe { crate::call_stack::CREATING_EFFECT = false };

        e
    }

    /// Creates a new `Effect`, wrapping the provided closure
    /// and running it immediately for dependency tracking.
    ///
    /// Returns an `Rc<dyn IEffect>` so the effect can be stored and shared
    /// as a non-generic trait object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{cell::Cell, rc::Rc};
    /// use reactive_cache::Effect;
    ///
    /// let counter = Rc::new(Cell::new(0));
    /// let c_clone = counter.clone();
    ///
    /// let effect = Effect::new(move || {
    ///     // This closure runs immediately
    ///     c_clone.set(c_clone.get() + 1);
    /// });
    ///
    /// assert_eq!(counter.get(), 1);
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new(f: F) -> Rc<dyn IEffect>
    where
        F: 'static,
    {
        Self::new_inner::<fn()>(f, None)
    }

    /// Creates a new `Effect` with an additional dependency initializer.
    ///
    /// This works like [`Effect::new`], but also runs the provided `deps` closure
    /// during the initial dependency collection phase.
    ///
    /// This is useful when your effect closure contains conditional logic
    /// (e.g. `if`/`match`), and you want to ensure that *all possible branches*
    /// have their dependencies tracked on the first run.
    ///
    /// Returns an `Rc<dyn IEffect>` so the effect can be stored and shared
    /// as a non-generic trait object.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{cell::Cell, rc::Rc};
    /// use reactive_cache::Effect;
    /// use reactive_macros::signal;
    ///
    /// signal!(static mut FLAG: bool = true;);
    /// signal!(static mut COUNTER: i32 = 10;);
    ///
    /// let result = Rc::new(Cell::new(0));
    /// let r_clone = result.clone();
    ///
    /// // Effect closure has a conditional branch
    /// let effect = Effect::new_with_deps(
    ///     move || {
    ///         match *FLAG_get() {
    ///             true => {}
    ///             false => {
    ///                 r_clone.set(*COUNTER_get());
    ///             }
    ///         }
    ///     },
    ///     // Explicitly declare both `FLAG` and `COUNTER` as dependencies
    ///     move || {
    ///         FLAG();
    ///         COUNTER();
    ///     },
    /// );
    ///
    /// assert_eq!(result.get(), 0); // runs with FLAG = true
    /// 
    /// // Changing `FLAG` to false will trigger the effect
    /// FLAG_set(false);
    /// assert_eq!(result.get(), 10);
    ///
    /// // Changing `COUNTER` still triggers the effect, even though
    /// // `FLAG` was true on the first run.
    /// COUNTER_set(20);
    /// assert_eq!(result.get(), 20);
    /// ```
    pub fn new_with_deps<D>(f: F, deps: D) -> Rc<dyn IEffect>
    where
        F: 'static,
        D: Fn() + 'static,
    {
        Self::new_inner(f, Some(deps))
    }
}

/// A non-generic trait for reactive effects.
///
/// `IEffect` serves as a type-erased trait for `Effect<F>` instances.
/// By implementing `IEffect`, an `Effect<F>` can be stored as `Rc<dyn IEffect>`
/// regardless of the specific closure type `F`. This allows the reactive system
/// to manage multiple effects uniformly without exposing the generic type.
pub trait IEffect {
    /// Runs the effect closure.
    ///
    /// Typically called by the reactive system when dependencies change.
    fn run(&self);
}

impl<F> IEffect for Effect<F>
where
    F: Fn(),
{
    fn run(&self) {
        (self.f)()
    }
}
