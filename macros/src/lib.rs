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

    let op_ident = format_ident!("{}_op", ident);
    let mut op_sig = sig.clone();
    op_sig.ident = op_ident.clone();
    op_sig
        .inputs
        .insert(0, parse_quote! { op: cache::MemoOperator });
    op_sig.output = parse_quote! { -> () };

    let expanded = quote! {
        #vis #sig
        where #output_ty: Clone + 'static
        {
            #op_ident(cache::MemoOperator::Memo(cache::Trace::Push));

            let key: cache::OperatorFunc = #op_ident;
            let rc = if let Some(rc) = cache::touch(key) {
                rc
            } else {
                let result: #output_ty = (|| #block)();
                cache::store_in_cache(key, result)
            };

            #op_ident(cache::MemoOperator::Memo(cache::Trace::Pop));

            (*rc).clone()
        }

        #vis #op_sig
        {
            static mut dependents: Vec<cache::OperatorFunc> = Vec::new();
            match op {
                cache::MemoOperator::Memo(cache::Trace::Push) => {
                    if let Some(last) = cache::call_stack::last() {
                        unsafe { dependents.push(last.clone()) };
                    }
                    cache::call_stack::push(#op_ident);
                },
                cache::MemoOperator::Memo(cache::Trace::Pop) => {
                    cache::call_stack::pop();
                },
                cache::MemoOperator::Pop => {
                    for dependent in unsafe { dependents.iter() } {
                        cache::remove_from_cache(*dependent);
                        dependent(cache::MemoOperator::Pop);
                    }
                },
            }
        }
    };

    expanded.into()
}
