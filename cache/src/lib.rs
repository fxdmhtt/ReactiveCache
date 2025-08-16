mod cache;
mod call_stack;

pub mod effect;
pub mod memo;
pub mod signal;

pub(crate) use cache::{remove_from_cache, store_in_cache, touch};
pub(crate) use call_stack::{creating_effect_peak, creating_effect_pop, creating_effect_push};
pub use effect::Effect;
pub use memo::Memo;
pub use signal::Signal;

pub(crate) trait Observable {
    fn invalidate(&'static self);
}
