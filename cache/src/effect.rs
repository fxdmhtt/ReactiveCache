use std::rc::Rc;

use crate::effect_stack::{effect_peak, effect_pop, effect_push};

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
    F: Fn() + 'static,
{
    f: F,
}

impl<F> Effect<F>
where
    F: Fn(),
{
    fn new_inner<D>(f: F, deps: Option<D>) -> Rc<dyn IEffect>
    where
        D: Fn() + 'static,
    {
        let e: Rc<dyn IEffect> = Rc::new(Effect { f });
        let w = Rc::downgrade(&e);

        // Dependency collection only at creation time
        effect_push(w.clone(), true);
        if let Some(deps) = &deps {
            deps();
        } else {
            e.run();
        }
        effect_pop(w.clone(), true);

        // If there is an additional dependency initializer,
        // the `Effect` needs to be run immediately
        // after dependency collection is completed.
        if deps.is_some() {
            run_untracked(&e);
        }

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
    pub fn new(f: F) -> Rc<dyn IEffect> {
        Self::new_inner::<fn()>(f, None)
    }

    /// Creates a new `Effect` with an additional dependency initializer.
    ///
    /// This works like [`Effect::new`], but requires a `deps` closure to be provided,
    /// which will be executed during the initial dependency collection phase.
    ///
    /// **Important:** Dependency tracking is performed **only when running `deps`**,
    /// not `f`. The closure `f` will still be executed when dependencies change,
    /// but its execution does **not** collect new dependencies.
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
    /// use reactive_macros::{ref_signal, signal};
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
    ///
    /// # Notes
    ///
    /// Any calls to `Effect` must be handled with care.
    ///
    /// After initialization, any calls to an `Effect` are completely dependent on
    /// run() , including Signal-triggered runs or dependency collection. However,
    /// these calls have different assumptions.
    ///
    /// Specifically, dependency collection for an `Effect` should be limited to
    /// its directly connected Signals. In this case, the `Effect`'s call chain
    /// conforms to the `Effect → Memo(s) → Signal(s)` model, which assumes that
    /// the `Effect` must be the start of the call chain. Any Signals linked to
    /// calls to other `Effect`s should not be collected, and runs triggered by
    /// `Signal`s should not be subject to dependency collection.
    fn run(&self);
}

impl<F> IEffect for Effect<F>
where
    F: Fn(),
{
    fn run(&self) {
        assert!(
            std::ptr::eq(&*effect_peak().unwrap().effect.upgrade().unwrap(), self),
            "`Effect` is not pushed onto the stack before being called."
        );

        (self.f)()
    }
}

pub(crate) fn run_untracked(e: &Rc<dyn IEffect>) {
    let w = Rc::downgrade(e);

    effect_push(w.clone(), false);
    e.run();
    effect_pop(w.clone(), false);
}
