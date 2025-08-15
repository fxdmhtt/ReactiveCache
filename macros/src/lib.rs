use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, ItemStatic, ReturnType, parse_macro_input};

#[proc_macro]
pub fn signal(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStatic);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let static_token = &item.static_token;
    let mutability = &item.mutability;
    let ident = &item.ident;
    let colon_token = &item.colon_token;
    let ty = &item.ty;
    let eq_token = &item.eq_token;
    let expr = &item.expr;
    let semi_token = &item.semi_token;

    let mutability = match mutability {
        syn::StaticMutability::Mut(_) => quote! { mut },
        syn::StaticMutability::None => quote! {},
        _ => panic!(),
    };

    let ty = quote! { once_cell::unsync::Lazy<std::rc::Rc<cache::Signal<#ty>>> };
    let expr = quote! { once_cell::unsync::Lazy::new(|| cache::Signal::new(Some(#expr))) };

    let expanded = quote! {
        #(#attrs)*
        #vis #static_token #mutability #ident #colon_token #ty #eq_token #expr #semi_token
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
