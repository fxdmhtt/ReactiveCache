use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use reactive_cache::{Effect, Memo, Signal};
use reactive_macros::{memo, ref_signal, signal};

// ----------------------
// Global signals
// ----------------------

// Non-Copy global string signal
ref_signal!(static mut GLOBAL_STR: String = "hello".to_string(););

// Copy global numeric signal
signal!(static mut GLOBAL_NUM: i32 = 10;);

// ----------------------
// Global memos
// ----------------------

#[memo]
pub fn get_global_number() -> i32 {
    // Getter for the global numeric signal
    GLOBAL_NUM()
}

#[memo]
pub fn get_global_string() -> String {
    // Getter for the global string signal
    GLOBAL_STR_get().clone()
}

// ----------------------
// ViewModel definition
// ----------------------

struct ViewModel {
    counter: Rc<Signal<i32>>,
    double: Rc<Memo<i32>>,
    effect: Rc<Effect>,
}

impl ViewModel {
    /// Create a new ViewModel with a local counter signal.
    /// The memo depends on the local counter and optionally reads the global numeric signal.
    /// The effect increments `effect_run_count` and reads the memo to establish reactive dependencies.
    fn new(initial_counter: i32, use_global_num: bool, effect_run_count: Rc<Cell<u32>>) -> Self {
        let counter = Signal::new(initial_counter);

        // Memo depends on `counter`, optionally reading the global numeric signal
        let double = Memo::new({
            let counter_clone = counter.clone();
            move || {
                let base = *counter_clone.get();
                let global_val = if use_global_num {
                    GLOBAL_NUM()
                } else {
                    0
                };
                base * 2 + global_val
            }
        });

        // Effect observes `double` and increments the counter
        let effect_run_count_clone = effect_run_count.clone();
        let double_for_effect = double.clone();
        let effect = Effect::new(move || {
            effect_run_count_clone.set(effect_run_count_clone.get() + 1);
            let _ = double_for_effect.get(); // subscribe to memo
        });

        Self {
            counter,
            double,
            effect,
        }
    }
}

// ----------------------
// Comprehensive integration test
// ----------------------

#[test]
fn test_global_and_vm_interaction_with_memory_cleanup() {
    // ----------------------
    // Arrange: run counters
    // ----------------------
    let global_effect_runs = Rc::new(Cell::new(0));
    let vm1_effect_runs = Rc::new(Cell::new(0));
    let vm2_effect_runs = Rc::new(Cell::new(0));

    // ----------------------
    // Global effect observing global memos
    // ----------------------
    let _global_effect = Effect::new({
        let runs = global_effect_runs.clone();
        move || {
            runs.set(runs.get() + 1);
            let _ = get_global_number();
            let _ = get_global_string();
        }
    });

    // Effect runs once upon creation
    assert_eq!(global_effect_runs.get(), 1);

    // ----------------------
    // Create two ViewModels
    // vm1 uses global numeric signal in its memo, vm2 does not
    // ----------------------
    let vm1 = ViewModel::new(1, true, vm1_effect_runs.clone());
    let vm2 = ViewModel::new(2, false, vm2_effect_runs.clone());

    // Keep weak references to check for leaks after dropping
    let vm1_counter_weak: Weak<Signal<i32>> = Rc::downgrade(&vm1.counter);
    let vm1_double_weak: Weak<Memo<i32>> = Rc::downgrade(&vm1.double);
    let vm1_effect_weak: Weak<Effect> = Rc::downgrade(&vm1.effect);

    let vm2_counter_weak: Weak<Signal<i32>> = Rc::downgrade(&vm2.counter);
    let vm2_double_weak: Weak<Memo<i32>> = Rc::downgrade(&vm2.double);
    let vm2_effect_weak: Weak<Effect> = Rc::downgrade(&vm2.effect);

    // Each VM effect runs once on creation
    assert_eq!(vm1_effect_runs.get(), 1);
    assert_eq!(vm2_effect_runs.get(), 1);

    // Global effect may have run once more depending on memo initialization
    assert_eq!(global_effect_runs.get(), 1);

    // ----------------------
    // Trigger updates and verify effects
    // ----------------------
    // Update vm1 counter -> vm1 effect should re-run
    vm1.counter.set(10);
    assert_eq!(vm1_effect_runs.get(), 2);
    assert_eq!(vm2_effect_runs.get(), 1); // vm2 unaffected

    // Update vm2 counter -> vm2 effect should re-run
    vm2.counter.set(7);
    assert_eq!(vm2_effect_runs.get(), 2);
    assert_eq!(vm1_effect_runs.get(), 2); // vm1 unaffected

    // Update global numeric signal -> global effect should re-run
    // vm1's memo depends on global numeric, so its effect should also run
    GLOBAL_NUM_set(100);
    assert_eq!(global_effect_runs.get(), 2);
    assert_eq!(vm1_effect_runs.get(), 3);
    assert_eq!(vm2_effect_runs.get(), 2); // vm2 does not depend on global

    // ----------------------
    // Verify memo values
    // vm1.double = counter*2 + global_num (10*2 + 100)
    // vm2.double = counter*2 (7*2)
    // ----------------------
    assert_eq!(vm1.double.get(), 10 * 2 + 100);
    assert_eq!(vm2.double.get(), 7 * 2);

    // ----------------------
    // Drop vm1 and check memory cleanup
    // ----------------------
    drop(vm1);
    assert!(vm1_counter_weak.upgrade().is_none());
    assert!(vm1_double_weak.upgrade().is_none());
    assert!(vm1_effect_weak.upgrade().is_none());

    // vm2 still alive
    assert!(vm2_counter_weak.upgrade().is_some());

    // ----------------------
    // Drop vm2 and verify cleanup
    // ----------------------
    drop(vm2);
    assert!(vm2_counter_weak.upgrade().is_none());
    assert!(vm2_double_weak.upgrade().is_none());
    assert!(vm2_effect_weak.upgrade().is_none());

    // ----------------------
    // Globals still alive after VMs are dropped
    // ----------------------
    let _n = get_global_number();
    let _s = get_global_string();

    // Sanity check on global values
    assert_eq!(GLOBAL_NUM(), 100);
    assert_eq!(get_global_number(), 100);
    assert_eq!(get_global_string(), "hello".to_string());
}