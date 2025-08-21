#![allow(static_mut_refs)]

use lru::LruCache;
use once_cell::unsync::Lazy;
use std::{any::Any, num::NonZeroUsize, rc::Rc};

use crate::Observable;

type CacheKey = *const dyn Observable;

const CACHE_CAP: usize = 128;

static mut CACHE: Lazy<LruCache<CacheKey, Rc<dyn Any>>> =
    Lazy::new(|| LruCache::new(NonZeroUsize::new(CACHE_CAP).unwrap()));

pub(crate) fn touch<T>(key: &'static dyn Observable) -> Option<Rc<T>>
where
    T: 'static,
{
    // When the Effect performs dependency calculations for the first time,
    // it must ignore the relevant cache,
    // otherwise the underlying Signal will not remember the Effect.
    if crate::effect_stack::effect_peak().is_some_and(|e| e.collecting) {
        remove_from_cache(key);
        return None;
    }

    unsafe { CACHE.get(&(key as _)) }
        .map(Rc::clone)
        .filter(|rc| rc.is::<T>())
        .map(|rc| unsafe { Rc::from_raw(Rc::into_raw(rc) as *const T) })
}

pub(crate) fn store_in_cache<T>(key: &'static dyn Observable, val: T) -> Rc<T>
where
    T: 'static,
{
    let rc = Rc::new(val);
    unsafe { CACHE.put(key, Rc::clone(&rc) as _) };
    rc
}

pub(crate) fn remove_from_cache(key: &'static dyn Observable) {
    unsafe { CACHE.pop(&(key as _)) };
}
