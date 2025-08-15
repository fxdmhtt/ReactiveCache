use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident, ItemFn, ItemStatic, ReturnType, parse_macro_input};

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

    let lazy_ty = quote! { once_cell::unsync::Lazy<std::rc::Rc<cache::Signal<#ty>>> };
    let expr = quote! { once_cell::unsync::Lazy::new(|| cache::Signal::new(Some(#expr))) };

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
    let ty = quote! { once_cell::unsync::Lazy<cache::Memo<#output_ty, fn() -> #output_ty>> };
    let expr = quote! { once_cell::unsync::Lazy::new(|| cache::Memo::new(|| #block)) };

    let expanded = quote! {
        static mut #ident: #ty = #expr;

        #vis #sig
        where #output_ty: Clone + 'static
        {
            unsafe { (*#ident).get() }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = match expr {
        Expr::Path(path) if path.path.get_ident().is_some() => {
            let ident = path.path.get_ident().unwrap();
            quote! {
                cache::Effect::new(#ident)
            }
        }
        Expr::Closure(closure) => {
            quote! {
                cache::Effect::new(#closure)
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
        where #output_ty: Eq + Clone + 'static
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
