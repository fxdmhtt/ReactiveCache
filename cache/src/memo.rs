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
///
/// # Examples
///
/// ## Basic usage
/// ```
/// use std::rc::Rc;
/// use reactive_cache::{Signal, Memo};
///
/// let counter = Rc::new(Signal::new(Some(1)));
/// let double = {
///     let counter = Rc::clone(&counter);
///     Memo::new({
///         let counter = Rc::new(counter);
///         move || *counter.get() * 2
///     })
/// };
///
/// assert_eq!(double.get(), 2);
/// counter.set(3);
/// assert_eq!(double.get(), 6);
/// ```
///
/// ## Using inside a struct
/// ```
/// use std::rc::Rc;
/// use reactive_cache::{Signal, Memo};
///
/// struct ViewModel {
///     counter: Rc<Signal<i32>>,
///     double: Rc<Memo<i32>>,
/// }
///
/// let counter = Rc::new(Signal::new(Some(1)));
/// let double = Memo::new({
///     let counter = counter.clone();
///     move || *counter.get() * 2
/// });
///
/// let vm = ViewModel { counter, double };
/// assert_eq!(vm.double.get(), 2);
/// vm.counter.set(4);
/// assert_eq!(vm.double.get(), 8);
/// ```
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
    /// Basic usage:
    /// ```
    /// use reactive_cache::Memo;
    ///
    /// let memo = Memo::new(|| 10);
    /// assert_eq!(memo.get(), 10);
    /// ```
    ///
    /// Using inside a struct:
    /// ```
    /// use std::rc::Rc;
    ///
    /// use reactive_cache::{Signal, Memo};
    ///
    /// struct ViewModel {
    ///     a: Rc<Signal<i32>>,
    ///     b: Rc<Signal<i32>>,
    ///     sum: Rc<Memo<i32>>,
    /// }
    ///
    /// // Construct signals
    /// let a = Rc::new(Signal::new(Some(2)));
    /// let b = Rc::new(Signal::new(Some(3)));
    ///
    /// // Construct a memo depending on `a` and `b`
    /// let sum = {
    ///     let a = a.clone();
    ///     let b = b.clone();
    ///     Memo::new(move || {
    ///         // `Signal::get()` will register dependencies automatically
    ///         *a.get() + *b.get()
    ///     })
    /// };
    ///
    /// let vm = ViewModel { a, b, sum };
    ///
    /// // Initial computation
    /// assert_eq!(vm.sum.get(), 5);
    ///
    /// // Update a signal → memo recomputes
    /// vm.a.set(10);
    /// assert_eq!(vm.sum.get(), 13);
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
