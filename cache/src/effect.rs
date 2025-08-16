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
        let e: Rc<dyn IEffect> = Rc::new(Effect { f });

        crate::creating_effect_push(Rc::downgrade(&e));
        e.run();
        crate::creating_effect_pop();

        e
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
