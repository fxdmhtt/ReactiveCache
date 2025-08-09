# ReactiveCache
**Cache smart, update only when it matters.**

[![Crates.io](https://img.shields.io/crates/v/reactivecache.svg)](https://crates.io/crates/reactivecache)
[![Docs.rs](https://docs.rs/reactivecache/badge.svg)](https://docs.rs/reactivecache)
[![License](https://img.shields.io/crates/l/reactivecache.svg)](LICENSE)

A lightweight, dependency-aware caching library with automatic invalidation and lazy recomputation.

---

**ReactiveCache** automatically tracks the relationships between computed values and their data sources.  
When a dependency becomes invalid, all cached results that rely on it are automatically invalidated.  
The next time you access a cached value, it will be recomputed lazily and stored again.

### Features
- **Automatic dependency tracking** – No manual dependency declarations.
- **Lazy recomputation** – Only recompute when accessed after invalidation.
- **Chained invalidation** – Changes ripple through the dependency graph.
- **Lightweight** – Minimal runtime overhead.

### Use cases
- Reactive data models
- Computed properties in UI frameworks
- Configuration and build systems
