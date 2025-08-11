use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, ReturnType, parse_macro_input, parse_quote};

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

    let _ident = format_ident!("_{}", sig.ident);
    let mut _sig = sig.clone();
    _sig.ident = _ident.clone();
    _sig.inputs
        .insert(0, parse_quote! { op: cache::MemoOperator });

    let expanded = quote! {
        #vis #sig
        where #output_ty: Clone + 'static
        {
            #_ident(cache::MemoOperator::Memo)
        }

        #vis #_sig
        where #output_ty: Clone + 'static
        {
            unsafe {
                let key = #ident as usize;
                let rc = if let Some(rc) = cache::touch(key) {
                    rc
                } else {
                    let result: #output_ty = (|| #block)();
                    cache::store_in_cache(key, result)
                };
                (*rc).clone()
            }
        }
    };

    expanded.into()
}
