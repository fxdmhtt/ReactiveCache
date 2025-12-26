#![allow(incomplete_features)]
#![feature(specialization)]

pub(crate) mod cache;
pub(crate) mod effect_stack;
pub(crate) mod memo_stack;
pub(crate) mod observable;

pub mod effect;
pub mod macros;
pub mod memo;
pub mod signal;

pub(crate) use cache::{remove_from_cache, store_in_cache, touch};
pub use effect::Effect;
pub(crate) use memo::IMemo;
pub use memo::Memo;
pub(crate) use observable::IObservable;
pub use signal::{Signal, SignalSetter};

pub use once_cell::unsync::Lazy;

pub mod prelude {
    pub use crate::Effect;
    pub use crate::Memo;
    pub use crate::Signal;
    pub use crate::SignalSetter as _;
}

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use reactive_macros::*;
