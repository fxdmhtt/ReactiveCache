use lru::LruCache;
use std::{any::Any, num::NonZeroUsize, rc::Rc};

use crate::OperatorFunc;

const CACHE_CAP: usize = 128;

static mut CACHE: Option<LruCache<OperatorFunc, Rc<dyn Any>>> = None;

fn cache() -> &'static mut LruCache<OperatorFunc, Rc<dyn Any>> {
    #[allow(static_mut_refs)]
    unsafe {
        CACHE.get_or_insert_with(|| LruCache::new(NonZeroUsize::new(CACHE_CAP).unwrap()))
    }
}

pub fn touch<T: 'static>(key: OperatorFunc) -> Option<Rc<T>> {
    cache()
        .get(&key)
        .map(Rc::clone)
        .filter(|rc| rc.is::<T>())
        .map(|rc| unsafe { Rc::from_raw(Rc::into_raw(rc) as *const T) })
}

pub fn store_in_cache<T: 'static>(key: OperatorFunc, val: T) -> Rc<T> {
    let rc = Rc::new(val);
    cache().put(key, Rc::clone(&rc) as _);
    rc
}

pub fn remove_from_cache(key: OperatorFunc) {
    cache().pop(&key);
}