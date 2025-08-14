pub enum Trace {
    Push,
    Pop,
}

pub enum MemoOperator {
    Memo(Trace),
    Pop,
}

pub mod cache;
pub mod call_stack;
pub mod memo;
pub mod signal;

pub use cache::{remove_from_cache, store_in_cache, touch};
pub use memo::Memo;
pub use signal::Signal;

pub trait Observable {
    fn invalidate(&'static self);
}
