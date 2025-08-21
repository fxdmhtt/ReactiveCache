pub(crate) mod cache;
pub(crate) mod call_stack;
pub(crate) mod effect_stack;

pub mod effect;
pub mod macros;
pub mod memo;
pub mod signal;

pub(crate) use cache::{remove_from_cache, store_in_cache, touch};
pub use effect::{Effect, IEffect};
pub use memo::Memo;
pub use signal::Signal;

pub(crate) trait Observable {
    fn invalidate(&'static self);
}

pub use once_cell::unsync::Lazy;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use reactive_macros::*;
