use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, ReturnType, parse_macro_input};

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

    let memo_ident = format_ident!("{}", ident.to_string().to_uppercase());

    let expanded = quote! {
        static mut #memo_ident: Option<cache::Memo<#output_ty, fn() -> #output_ty>> = None;

        #vis #sig
        where #output_ty: Clone + 'static
        {
            #[allow(static_mut_refs)]
            unsafe { &mut #memo_ident }.get_or_insert_with(|| cache::Memo::new(|| #block)).get()
        }
    };

    expanded.into()
}
