/// Creates a reactive effect from a closure (and optionally a dependency collector).
///
/// The `effect!` macro is a convenient wrapper around
/// [`reactive_cache::Effect::new`] and [`reactive_cache::Effect::new_with_deps`].
/// It allows you to quickly register a reactive effect that automatically tracks
/// dependencies and re-runs when they change.
///
/// # Forms
///
/// - `effect!(f)`  
///   Equivalent to calling [`Effect::new(f)`]. In this form, dependencies are
///   automatically tracked while executing `f`.
///
/// - `effect!(f, deps)`  
///   Equivalent to calling [`Effect::new_with_deps(f, deps)`]. In this form,
///   **dependency tracking is performed only when running `deps`**, not `f`.
///   The closure `f` will still be executed when dependencies change, but its
///   execution does **not** collect new dependencies.
///
/// # Requirements
///
/// - `f` must be a closure or function pointer that takes no arguments and returns `()`.
/// - `deps` (if provided) must also be a closure or function pointer taking no arguments and returning `()`.
///
/// # Examples
///
/// ```rust
/// use std::{cell::Cell, rc::Rc};
/// use reactive_cache::effect;
/// use reactive_cache::prelude::*;
/// use reactive_macros::signal;
///
/// signal!(static mut A: i32 = 1;);
///
/// // Track effect runs
/// let counter = Rc::new(Cell::new(0));
/// let counter_clone = counter.clone();
///
/// // `effect!(f)` form
/// let e = effect!(move || {
///     let _ = A().get();           // reading the signal
///     counter_clone.set(counter_clone.get() + 1); // increment effect counter
/// });
///
/// let ptr = Rc::into_raw(e); // actively leak to avoid implicitly dropping the effect
///
/// // Effect runs immediately upon creation
/// assert_eq!(counter.get(), 1);
///
/// // Changing A triggers the effect again
/// assert!(A().set(10));
/// assert_eq!(counter.get(), 2);
///
/// // Setting the same value does NOT trigger the effect
/// assert!(!A().set(10));
/// assert_eq!(counter.get(), 2);
///
/// // `effect!(f, deps)` form
/// let _ = effect!(
///     || println!("effect body"),
///     || println!("dependency collector")
/// );
/// ```
///
/// # SAFETY
///
/// The macro internally uses [`reactive_cache::Effect`], which relies on
/// `static` tracking and is **not thread-safe**. Only use in single-threaded contexts.
///
/// # Warning
///
/// **Do not set any signal that is part of the same effect chain.**
///
/// Effects automatically run whenever one of their dependent signals changes.
/// If an effect modifies a signal that it (directly or indirectly) observes,
/// it creates a circular dependency. This can lead to:
/// - an infinite loop of updates, or
/// - conflicting updates that the system cannot resolve.
///
/// In the general case, it is impossible to automatically determine whether
/// such an effect will ever terminate—this is essentially a version of the
/// halting problem. Therefore, you must ensure manually that effects do not
/// update signals within their own dependency chain.
#[macro_export]
macro_rules! effect {
    ($f:expr) => {
        $crate::Effect::new($f)
    };
    ($f:expr, $f2:expr) => {
        $crate::Effect::new_with_deps($f, $f2)
    };
}
