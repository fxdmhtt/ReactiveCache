use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, ItemStatic, ReturnType, parse_macro_input};

/// Wraps a `static mut` variable as a reactive global signal.
///
/// The `signal!` macro transforms a `static mut` variable into a `reactive_cache::Signal`,
/// and generates a **function with the same name as the variable** that returns a
/// `&'static Rc<Signal<T>>`. You can then call `.get()` to read the value or `.set(value)` to update it.
///
/// # Requirements
///
/// - Supports only `static mut` variables.
/// - Type `T` must implement `Eq`.
///
/// # Examples
///
/// ```rust
/// use reactive_cache::prelude::*;
/// use reactive_macros::signal;
///
/// signal!(static mut A: i32 = 10;);
///
/// assert_eq!(*A().get(), 10);
/// assert!(A().set(20));
/// assert_eq!(*A().get(), 20);
/// assert!(!A().set(20)); // No change
///
/// signal!(static mut B: String = "hello".to_string(););
///
/// assert_eq!(*B().get(), "hello");
/// assert!(B().set("world".to_string()));
/// assert_eq!(*B().get(), "world");
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

    let vis = &item.vis;
    let ident = &item.ident;
    let ty = &item.ty;
    let expr = &item.expr;

    let lazy_ty = quote! { reactive_cache::Lazy<std::rc::Rc<reactive_cache::Signal<#ty>>> };
    let expr = quote! { reactive_cache::Lazy::new(|| reactive_cache::Signal::new(#expr)) };

    let expanded = quote! {
        #[allow(non_snake_case)]
        #vis fn #ident() -> &'static std::rc::Rc<reactive_cache::Signal<#ty>> {
            static mut #ident: #lazy_ty = #expr;
            unsafe { &*#ident }
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
/// use reactive_cache::prelude::*;
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
/// #[memo]
/// pub fn get_string() -> String {
///     "Hello, World!".to_string()
/// }
///
/// fn main() {
///     // First call computes and caches the value
///     assert_eq!(get_number(), 42);
///     // Subsequent calls return the cached value without re-running the block
///     assert_eq!(get_number(), 42);
///
///     assert_eq!(get_string(), "Hello, World!");
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
    let ty = quote! { reactive_cache::Lazy<std::rc::Rc<reactive_cache::Memo<#output_ty>>> };
    let expr = quote! { reactive_cache::Lazy::new(|| reactive_cache::Memo::new(|| #block)) };

    let expanded = quote! {
        #vis #sig {
            static mut #ident: #ty = #expr;
            unsafe { #ident.get() }
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
/// use reactive_cache::prelude::*;
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
