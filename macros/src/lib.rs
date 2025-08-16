use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident, ItemFn, ItemStatic, ReturnType, parse_macro_input};

/// Wraps a `static mut` variable as a reactive signal (similar to a property)
/// with getter and setter functions.
///
/// The `signal!` macro transforms a `static mut` variable into a `reactive_cache::Signal`,
/// and automatically generates:
/// 1. A `_get()` function to read the value.
/// 2. A `_set(value)` function to write the value (returns `true` if changed).
/// 3. A function with the same name as the variable to simplify access (calls `_get()`).
///
/// # Requirements
///
/// - The macro currently supports only `static mut` variables.
/// - The variable type must implement `Eq + Default`.
///
/// # Examples
///
/// ```rust
/// use std::cell::Cell;
/// use reactive_macros::signal;
///
/// signal!(static mut A: i32 = 10;);
///
/// assert_eq!(A(), 10);
/// assert_eq!(A_get(), 10);
/// assert!(A_set(20));
/// assert_eq!(A(), 20);
/// assert!(!A_set(20)); // No change
/// ```
///
/// # SAFETY
///
/// This macro wraps `static mut` variables internally, so it **is not thread-safe**.
/// It should be used only in single-threaded contexts.
///
/// # Warning
///
/// **Do not set any signal that is part of the same effect chain.**
///
/// Effects automatically run whenever one of their dependent signals changes.
/// If an effect modifies a signal that it (directly or indirectly) observes,
/// it creates a circular dependency. This can lead to:
/// - an infinite loop of updates, or
/// - conflicting updates that the system cannot resolve.
///
/// In the general case, it is impossible to automatically determine whether
/// such an effect will ever terminate—this is essentially a version of the
/// halting problem. Therefore, you must ensure manually that effects do not
/// update signals within their own dependency chain.
#[proc_macro]
pub fn signal(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStatic);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let static_token = &item.static_token;
    let _mutability = &item.mutability;
    let ident = &item.ident;
    let colon_token = &item.colon_token;
    let ty = &item.ty;
    let eq_token = &item.eq_token;
    let expr = &item.expr;
    let semi_token = &item.semi_token;

    let mutability = match &item.mutability {
        syn::StaticMutability::Mut(_) => quote! { mut },
        syn::StaticMutability::None => quote! {},
        _ => {
            return syn::Error::new_spanned(&item.mutability, "Mutability not supported")
                .to_compile_error()
                .into();
        }
    };

    let ident_p = format_ident!("_{}", ident.to_string().to_uppercase());
    let ident_get = format_ident!("{}_get", ident);
    let ident_set = format_ident!("{}_set", ident);
    let ident_fn = format_ident!("{}", ident);

    let lazy_ty = quote! { once_cell::unsync::Lazy<reactive_cache::Signal<#ty>> };
    let expr = quote! { once_cell::unsync::Lazy::new(|| reactive_cache::Signal::new(Some(#expr))) };

    let expanded = quote! {
        #(#attrs)*
        #vis #static_token #mutability #ident_p #colon_token #lazy_ty #eq_token #expr #semi_token

        #[allow(non_snake_case)]
        pub fn #ident_get() -> #ty {
            unsafe { *#ident_p.get() }
        }

        #[allow(non_snake_case)]
        pub fn #ident_set(value: #ty) -> bool {
            unsafe { #ident_p.set(value) }
        }

        #[allow(non_snake_case)]
        pub fn #ident_fn() -> #ty {
            #ident_get()
        }
    };

    expanded.into()
}

/// Turns a zero-argument function into a memoized, reactive computation.
///
/// The `#[memo]` attribute macro transforms a function into a static
/// `reactive_cache::Memo`, which:
/// 1. Computes the value the first time the function is called.
/// 2. Caches the result for future calls.
/// 3. Automatically tracks reactive dependencies if used inside `Signal` or other reactive contexts.
///
/// # Requirements
///
/// - The function must have **no parameters**.
/// - The function must return a value (`-> T`), which must implement `Clone`.
///
/// # Examples
///
/// ```rust
/// use reactive_macros::memo;
///
/// #[memo]
/// pub fn get_number() -> i32 {
///     // The first call sets INVOKED to true
///     static mut INVOKED: bool = false;
///     assert!(!unsafe { INVOKED });
///     unsafe { INVOKED = true };
///
///     42
/// }
///
/// fn main() {
///     // First call computes and caches the value
///     assert_eq!(get_number(), 42);
///     // Subsequent calls return the cached value without re-running the block
///     assert_eq!(get_number(), 42);
/// }
/// ```
///
/// # SAFETY
///
/// This macro uses a `static mut` internally, so it **is not thread-safe**.
/// It is intended for single-threaded usage only. Accessing the memo from
/// multiple threads concurrently can cause undefined behavior.
#[proc_macro_attribute]
pub fn memo(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let ident = &func.sig.ident;

    let output_ty = match &sig.output {
        ReturnType::Type(_, ty) => ty.clone(),
        _ => {
            return syn::Error::new_spanned(&sig.output, "Functions must have a return value")
                .to_compile_error()
                .into();
        }
    };

    if !sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &sig.inputs,
            "The memo macro can only be used with `get` function without any parameters.",
        )
        .to_compile_error()
        .into();
    }

    let ident = format_ident!("{}", ident.to_string().to_uppercase());
    let ty =
        quote! { once_cell::unsync::Lazy<reactive_cache::Memo<#output_ty, fn() -> #output_ty>> };
    let expr = quote! { once_cell::unsync::Lazy::new(|| reactive_cache::Memo::new(|| #block)) };

    let expanded = quote! {
        static mut #ident: #ty = #expr;

        #vis #sig
        where #output_ty: Clone
        {
            unsafe { (*#ident).get() }
        }
    };

    expanded.into()
}

/// Creates a reactive effect from a closure or function pointer.
///
/// The `effect!` procedural macro is a convenient wrapper around `reactive_cache::Effect::new`.
/// It allows you to quickly register a reactive effect that automatically tracks
/// dependencies and re-runs when they change.
///
/// # Requirements
///
/// - The argument must be either:
///   1. A closure (e.g., `|| { ... }`), or  
///   2. A function pointer / function name with zero arguments.
/// - The closure or function must return `()` (no return value required).
///
/// # Examples
///
/// ```rust
/// use std::{cell::Cell, rc::Rc};
/// use reactive_macros::{effect, signal};
///
/// signal!(static mut A: i32 = 1;);
///
/// // Track effect runs
/// let counter = Rc::new(Cell::new(0));
/// let counter_clone = counter.clone();
///
/// let e = effect!(move || {
///     let _ = A();           // reading the signal
///     counter_clone.set(counter_clone.get() + 1); // increment effect counter
/// });
///
/// let ptr = Rc::into_raw(e); // actively leak to avoid implicitly dropping the effect
///
/// // Effect runs immediately upon creation
/// assert_eq!(counter.get(), 1);
///
/// // Changing A triggers the effect again
/// assert!(A_set(10));
/// assert_eq!(counter.get(), 2);
///
/// // Setting the same value does NOT trigger the effect
/// assert!(!A_set(10));
/// assert_eq!(counter.get(), 2);
/// ```
///
/// # SAFETY
///
/// The macro internally uses `reactive_cache::Effect`, which relies on
/// `static` tracking and is **not thread-safe**. Only use in single-threaded contexts.
///
/// # Warning
///
/// **Do not set any signal that is part of the same effect chain.**
///
/// Effects automatically run whenever one of their dependent signals changes.
/// If an effect modifies a signal that it (directly or indirectly) observes,
/// it creates a circular dependency. This can lead to:
/// - an infinite loop of updates, or
/// - conflicting updates that the system cannot resolve.
///
/// In the general case, it is impossible to automatically determine whether
/// such an effect will ever terminate—this is essentially a version of the
/// halting problem. Therefore, you must ensure manually that effects do not
/// update signals within their own dependency chain.
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = match expr {
        Expr::Path(path) if path.path.get_ident().is_some() => {
            let ident = path.path.get_ident().unwrap();
            quote! {
                reactive_cache::Effect::new(#ident)
            }
        }
        Expr::Closure(closure) => {
            quote! {
                reactive_cache::Effect::new(#closure)
            }
        }
        _ => {
            return syn::Error::new_spanned(&expr, "Expected a variable name or a closure")
                .to_compile_error()
                .into();
        }
    };

    expanded.into()
}

/// Evaluates a zero-argument function and optionally reports when the value changes.
///
/// The `#[evaluate(print_fn)]` attribute macro transforms a function into a reactive
/// evaluator that:
/// 1. Computes the function result on each call.
/// 2. Compares it with the previously computed value.
/// 3. If the value is unchanged, calls the specified print function with a message.
///
/// # Requirements
///
/// - The function must have **no parameters**.
/// - The function must return a value (`-> T`), which must implement `Eq + Clone`.
/// - The print function (e.g., `print`) must be a callable accepting a `String`.
///
/// # Examples
///
/// ```rust
/// use reactive_macros::evaluate;
///
/// fn print(msg: String) {
///     println!("{}", msg);
/// }
///
/// #[evaluate(print)]
/// pub fn get_number() -> i32 {
///     42
/// }
///
/// fn main() {
///     // First call computes the value
///     assert_eq!(get_number(), 42);
///     // Second call compares with previous; prints message since value didn't change
///     assert_eq!(get_number(), 42);
/// }
/// ```
///
/// # SAFETY
///
/// This macro uses a `static mut` internally to store the previous value,
/// so it **is not thread-safe**. It should only be used in single-threaded contexts.
#[proc_macro_attribute]
pub fn evaluate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let print = parse_macro_input!(attr as Ident);
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let ident = &func.sig.ident;

    let output_ty = match &sig.output {
        ReturnType::Type(_, ty) => ty.clone(),
        _ => {
            return syn::Error::new_spanned(&sig.output, "Functions must have a return value")
                .to_compile_error()
                .into();
        }
    };

    if !sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &sig.inputs,
            "The memo macro can only be used with `get` function without any parameters.",
        )
        .to_compile_error()
        .into();
    }

    let option_ty = quote! { Option<#output_ty> };
    let ident = ident.to_string();

    let expanded = quote! {
        #vis #sig
        where #output_ty: Eq + Clone
        {
            let new: #output_ty = (|| #block)();

            static mut VALUE: #option_ty = None;
            if let Some(old) = unsafe { VALUE } && old == new {
                #print(format!("Evaluate: {} not changed, still {:?}\n", #ident, new));
            }
            unsafe { VALUE = Some(new.clone()) };

            new
        }
    };

    expanded.into()
}
