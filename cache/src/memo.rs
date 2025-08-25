use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{IObservable, memo_stack, store_in_cache, touch};

/// A memoized reactive computation that caches its result and tracks dependencies.
///
/// `Memo<T>` behaves similarly to a computed property: it stores the result of a closure
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
pub struct Memo<T> {
    f: Box<dyn Fn() -> T>,
    dependents: RefCell<Vec<Weak<dyn IMemo>>>,
    /// A self-referential weak pointer, set during construction with `Rc::new_cyclic`.
    /// Used to upgrade to `Rc<Memo<T>>` and then coerce into `Rc<dyn IMemo>` when needed.
    weak: Weak<Memo<T>>,
}

impl<T> Memo<T> {
    /// Creates a new `Memo` wrapping the provided closure.
    ///
    /// # Requirements
    /// - `T` must be `'static`, because the value is stored in global cache.
    /// - The closure must be `'static` as well.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Memo;
    ///
    /// let memo = Memo::new(|| 10);
    /// assert_eq!(memo.get(), 10);
    /// ```
    pub fn new(f: impl Fn() -> T + 'static) -> Rc<Self>
    where
        T: 'static,
    {
        Rc::new_cyclic(|weak| Memo {
            f: Box::new(f),
            dependents: vec![].into(),
            weak: weak.clone(),
        })
    }

    /// Returns the memoized value, recomputing it only if necessary.
    ///
    /// During the computation, dependencies are tracked for reactive updates.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactive_cache::Memo;
    ///
    /// let memo = Memo::new(|| 5);
    /// assert_eq!(memo.get(), 5);
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone + 'static,
    {
        self.dependency_collection();

        memo_stack::push(self.weak.clone());

        let rc = if let Some(this) = self.weak.upgrade() {
            let key: Rc<dyn IMemo> = this.clone();
            if let Some(rc) = touch(&key) {
                rc
            } else {
                let result: T = (self.f)();
                store_in_cache(&key, result)
            }
        } else {
            unreachable!()
        };

        memo_stack::pop();

        (*rc).clone()
    }
}

impl<T> IObservable for Memo<T> {
    fn dependents(&self) -> &RefCell<Vec<Weak<dyn IMemo>>> {
        &self.dependents
    }
}

/// Internal marker trait for all memoized computations.
/// Used for type erasure when storing heterogeneous `Memo<T>` in caches.
pub(crate) trait IMemo: IObservable {}

impl<T> IMemo for Memo<T> {}
