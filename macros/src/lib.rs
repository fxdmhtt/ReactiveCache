use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, ReturnType, parse_macro_input};

#[proc_macro_attribute]
pub fn signal(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let expanded = quote! {
        static mut #ident: once_cell::unsync::Lazy<cache::Signal<#output_ty, fn() -> #output_ty>> = once_cell::unsync::Lazy::new(|| cache::Signal::new(|| #block));

        #vis #sig
        where #output_ty: Clone + 'static
        {
            unsafe { (*#ident).get() }
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

    let expanded = quote! {
        static mut #ident: once_cell::unsync::Lazy<cache::Memo<#output_ty, fn() -> #output_ty>> = once_cell::unsync::Lazy::new(|| cache::Memo::new(|| #block));

        #vis #sig
        where #output_ty: Clone + 'static
        {
            unsafe { (*#ident).get() }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn effect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let ident = &func.sig.ident;

    if let ReturnType::Type(_, _) = &sig.output {
        return syn::Error::new_spanned(&sig.output, "Functions must have no return value")
            .to_compile_error()
            .into();
    }

    if !sig.inputs.is_empty() {
        return syn::Error::new_spanned(
            &sig.inputs,
            "The memo macro can only be used with `get` function without any parameters.",
        )
        .to_compile_error()
        .into();
    }

    let ident = format_ident!("{}", ident.to_string().to_uppercase());

    let expanded = quote! {
        static mut #ident: once_cell::unsync::Lazy<std::rc::Rc<cache::Effect>> = once_cell::unsync::Lazy::new(|| cache::Effect::new(|| #block));

        #vis #sig
        {
            unsafe { (*#ident).run() }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn effect_init(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as Ident);
    let ident = proc_macro2::Ident::new(&ident.to_string().to_uppercase(), ident.span());

    let expanded = quote! {
        once_cell::unsync::Lazy::force(&#ident)
    };

    expanded.into()
}
