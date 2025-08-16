use std::cell::RefCell;

use crate::{Observable, call_stack, remove_from_cache, store_in_cache, touch};

/// A memoized reactive computation that caches its result and tracks dependencies.
///
/// `Memo<T, F>` behaves similarly to a computed property: it stores the result of a closure
/// and only recomputes when its dependencies change. Other signals or effects that access
/// the memo will automatically be tracked.
///
/// In short:
/// - Like a computed property: returns a cached value derived from other signals.
/// - Adds tracking: recomputes only when dependencies are invalidated.
///
/// # Type Parameters
///
/// - `T`: The result type of the computation. Must implement `Clone`.
/// - `F`: The closure type that computes the value. Must implement `Fn() -> T`.
pub struct Memo<T, F>
where
    F: Fn() -> T,
{
    f: F,
    dependents: RefCell<Vec<&'static dyn Observable>>,
}

impl<T, F> Observable for Memo<T, F>
where
    F: Fn() -> T,
{
    fn invalidate(&'static self) {
        remove_from_cache(self);
        self.dependents.borrow().iter().for_each(|d| d.invalidate());
    }
}

impl<T, F> Memo<T, F>
where
    F: Fn() -> T,
{
    /// Creates a new `Memo` wrapping the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Memo;
    ///
    /// let memo = Memo::new(|| 10);
    /// ```
    pub fn new(f: F) -> Self {
        Memo {
            f,
            dependents: vec![].into(),
        }
    }

    /// Returns the memoized value, recomputing it only if necessary.
    ///
    /// During the computation, dependencies are tracked for reactive updates.
    ///
    /// # Examples
    ///
    /// ```
    /// use once_cell::unsync::Lazy;
    /// use reactive_cache::Memo;
    ///
    /// static mut MEMO: Lazy<Memo<i32, fn() -> i32>> = Lazy::new(|| Memo::new(|| 5));
    /// assert_eq!(unsafe { (*MEMO).get() }, 5);
    /// ```
    pub fn get(&'static self) -> T
    where
        T: Clone,
    {
        if let Some(last) = call_stack::last()
            && !self
                .dependents
                .borrow()
                .iter()
                .any(|d| std::ptr::eq(*d, *last))
        {
            self.dependents.borrow_mut().push(*last);
        }

        call_stack::push(self);

        let rc = if let Some(rc) = touch(self) {
            rc
        } else {
            let result: T = (self.f)();
            store_in_cache(self, result)
        };

        call_stack::pop();

        (*rc).clone()
    }
}
