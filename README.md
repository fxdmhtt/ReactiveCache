# ReactiveCache
**Cache smart, update only when it matters.**

[![Crates.io](https://img.shields.io/crates/v/reactivecache.svg)](https://crates.io/crates/reactivecache)
[![Docs.rs](https://docs.rs/reactivecache/badge.svg)](https://docs.rs/reactivecache)
[![License](https://img.shields.io/crates/l/reactivecache.svg)](LICENSE)

A lightweight, dependency-aware memoization library with automatic invalidation and lazy recomputation.

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

---

### Signal-Memo-Effect Relationship

ReactiveCache is structured around a clear three-tier reactive model:

Signal (atomic mutable)
|
+--> Memo (derived computation, cached value, lazy)
|
+--> Effect (side-effectful computation, eager)

1. **Signal**  
   Signals are the source of truth in the system. They hold raw values that can change over time.  
   Updating a signal automatically marks any dependent computations as potentially stale.

2. **Memo**  
   Memos are derived, computed values that depend on one or more signals (or other memos).  
   When a signal changes, all memos that depend on it are invalidated.  
   Accessing a memo after invalidation triggers lazy recomputation, ensuring that cached values are always consistent.

3. **Effect**  
   Effects are side-effectful computations that run automatically whenever their dependencies change.  
   They subscribe to signals and memos, reacting to updates without manual intervention.  
   Effects do not produce cached values; instead, they propagate changes outward (e.g., updating UI, logging, or triggering external events).

**Dependency Flow:**  

- **Signals** update their dependents (memos/effects).  
- **Memos** recompute lazily when accessed, maintaining up-to-date derived values.  
- **Effects** automatically react whenever any dependency they use changes.

This separation ensures efficient and predictable propagation: cached computations are only recomputed when needed, while side effects happen immediately when dependencies change.

This three-level model ensures that changes propagate efficiently, only recomputing what is necessary, and automatically triggering side-effects in a controlled and predictable way.
