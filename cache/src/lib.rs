pub enum Trace {
    Push,
    Pop,
}

pub enum MemoOperator {
    Memo(Trace),
    Pop,
}

pub type OperatorFunc = fn(MemoOperator);

pub mod cache;
pub mod call_stack;

pub use cache::{remove_from_cache, store_in_cache, touch};
