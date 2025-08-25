use std::{cell::RefCell, rc::Weak};

use crate::{IMemo, memo_stack, remove_from_cache};

pub(crate) trait IObservable {
    fn dependents(&self) -> &RefCell<Vec<Weak<dyn IMemo>>>;

    /// Invalidates all dependent observables.
    fn invalidate(&self) {
        self.dependents().borrow_mut().retain(|d| {
            if let Some(d) = d.upgrade() {
                remove_from_cache(&d);
                d.invalidate();
                true
            } else {
                false
            }
        });
    }

    /// Track observables in the call stack
    fn dependency_collection(&self) {
        if let Some(last) = memo_stack::last()
            && !self
                .dependents()
                .borrow()
                .iter()
                .any(|d| Weak::ptr_eq(d, last))
        {
            self.dependents().borrow_mut().push(last.clone());
        }
    }
}
