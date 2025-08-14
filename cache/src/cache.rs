use lru::LruCache;
use once_cell::unsync::Lazy;
use std::{any::Any, num::NonZeroUsize, rc::Rc};

use crate::Observable;

type CacheKey = *const dyn Observable;

const CACHE_CAP: usize = 128;

static mut CACHE: Lazy<LruCache<CacheKey, Rc<dyn Any>>> =
    Lazy::new(|| LruCache::new(NonZeroUsize::new(CACHE_CAP).unwrap()));

pub fn touch<T: 'static>(key: &'static dyn Observable) -> Option<Rc<T>> {
    unsafe { (*CACHE).get(&(key as _)) }
        .map(Rc::clone)
        .filter(|rc| rc.is::<T>())
        .map(|rc| unsafe { Rc::from_raw(Rc::into_raw(rc) as *const T) })
}

pub fn store_in_cache<T: 'static>(key: &'static dyn Observable, val: T) -> Rc<T> {
    let rc = Rc::new(val);
    unsafe { (*CACHE).put(key, Rc::clone(&rc) as _) };
    rc
}

pub fn remove_from_cache(key: &'static dyn Observable) {
    unsafe { (*CACHE).pop(&(key as _)) };
}
