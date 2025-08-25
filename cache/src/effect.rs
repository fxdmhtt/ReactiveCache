use std::rc::Rc;

use crate::effect_stack::{effect_peak, effect_pop, effect_push};

/// A reactive effect that runs a closure whenever its dependencies change.
///
/// `Effect` behaves similarly to an "event listener" or a callback,
/// but it is automatically tied to any signals or memos it reads during execution.
/// When those dependencies change, the effect will re-run.
///
/// Note: The closure runs **immediately upon creation** via [`Effect::new`],
/// so the effect is always initialized with an up-to-date value.
///
/// In short:
/// - Like a callback: wraps a closure and runs it.
/// - Adds tracking: automatically re-runs when dependent signals change.
/// - Runs once immediately at creation.
///
/// # Examples
///
/// ## Basic usage
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
///
/// ## Using inside a struct
/// ```
/// use std::{rc::Rc, cell::Cell};
/// use reactive_cache::{Signal, Memo, Effect};
///
/// struct ViewModel {
///     counter: Rc<Signal<i32>>,
///     double: Rc<Memo<i32>>,
///     effect: Rc<Effect>,
///     run_count: Rc<Cell<u32>>,
/// }
///
/// let counter = Signal::new(1);
/// let double = Memo::new({
///     let counter = counter.clone();
///     move || *counter.get() * 2
/// });
///
/// let run_count = Rc::new(Cell::new(0));
/// let run_count_clone = run_count.clone();
///
/// let effect = Effect::new({
///     let double = double.clone();
///     move || {
///         run_count_clone.set(run_count_clone.get() + 1);
///         let _ = double.get();
///     }
/// });
///
/// let vm = ViewModel {
///     counter: counter.clone(),
///     double: double.clone(),
///     effect: effect,
///     run_count: run_count.clone(),
/// };
///
/// assert_eq!(run_count.get(), 1);
/// vm.counter.set(4);
/// assert_eq!(run_count.get(), 2);
/// ```
pub struct Effect {
    f: Box<dyn Fn()>,
}

impl Effect {
    /// Creates a new `Effect`, wrapping the provided closure
    /// and running it immediately for dependency tracking.
    ///
    /// Returns an `Rc<Effect>` so the effect can be stored and shared
    /// as a non-generic type.
    ///
    /// # Examples
    ///
    /// ## Basic usage
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
    ///
    /// ## Using inside a struct
    /// ```
    /// use std::rc::Rc;
    /// use reactive_cache::{Signal, Memo, Effect};
    ///
    /// struct ViewModel {
    ///     counter: Rc<Signal<i32>>,
    ///     double: Rc<Memo<i32>>,
    ///     effect: Rc<Effect>,
    /// }
    ///
    /// let counter = Signal::new(1);
    /// let double = Memo::new({
    ///     let counter = counter.clone();
    ///     move || *counter.get() * 2
    /// });
    ///
    /// let vm = ViewModel {
    ///     counter: counter.clone(),
    ///     double: double.clone(),
    ///     effect: Effect::new({
    ///         let double = double.clone();
    ///         move || println!("Double is {}", double.get())
    ///     }),
    /// };
    ///
    /// counter.set(3);
    /// assert_eq!(double.get(), 6);
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new(f: impl Fn() + 'static) -> Rc<Effect> {
        let e: Rc<Effect> = Rc::new(Effect { f: Box::new(f) });
        let w = Rc::downgrade(&e);

        // Dependency collection only at creation time
        effect_push(w.clone(), true);
        e.run();
        effect_pop(w.clone(), true);

        e
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
    /// Returns an `Rc<Effect>` so the effect can be stored and shared
    /// as a non-generic type.
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
    pub fn new_with_deps(f: impl Fn() + 'static, deps: impl Fn()) -> Rc<Effect> {
        let e: Rc<Effect> = Rc::new(Effect { f: Box::new(f) });
        let w = Rc::downgrade(&e);

        // Dependency collection only at creation time
        effect_push(w.clone(), true);
        deps();
        effect_pop(w.clone(), true);

        // If there is an additional dependency initializer,
        // the `Effect` needs to be run immediately
        // after dependency collection is completed.
        run_untracked(&e);

        e
    }

    /// Runs the effect closure.
    ///
    /// Typically called by the reactive system when dependencies change.
    ///
    /// # Notes
    ///
    /// After initialization, any call to an `Effect` must go through `run()`.
    /// Since the preconditions for executing `run()` differ depending on context
    /// (e.g. dependency collection vs. signal-triggered updates), such calls
    /// must be handled with care.
    ///
    /// Dependency collection for an `Effect` should be limited to its directly
    /// connected signals. The intended call chain is:
    ///
    /// `Effect → Memo(s) → Signal(s)`
    ///
    /// In this model, the `Effect` must always be the root of the chain.
    /// Other `Effect`s should not be tracked as dependencies, and runs triggered
    /// by signals should not themselves cause further dependency collection.
    fn run(&self) {
        assert!(
            std::ptr::eq(&*effect_peak().unwrap().effect.upgrade().unwrap(), self),
            "`Effect` is not pushed onto the stack before being called."
        );

        (self.f)()
    }
}

pub(crate) fn run_untracked(e: &Rc<Effect>) {
    let w = Rc::downgrade(e);

    effect_push(w.clone(), false);
    e.run();
    effect_pop(w.clone(), false);
}
