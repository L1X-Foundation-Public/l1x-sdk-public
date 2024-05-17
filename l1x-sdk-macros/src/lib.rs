use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemImpl;
use syn::Signature;
use syn::Visibility;

fn input_struct_deser(sig: &Signature) -> TokenStream2 {
    let mut fields = TokenStream2::new();
    for arg in &sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => todo!(),
            syn::FnArg::Typed(typed) => {
                let ident = &typed.pat;
                let ty = &typed.ty;
                fields.extend(quote! {
                    #ident: #ty,
                });
            }
        }
    }
    quote! {
        #[derive(serde::Deserialize)]
        struct Input {
            #fields
        }
    }
}

/// Walks over public methods and generates wrappers for each method it finds.
///
/// The generated wrapper reads method arguments [`l1x_sdk::input`], deserializes them, and calls the original method.
/// When the original method returns, the wrapper serializes the returned value and writes the serialized value with `l1x_sdk::output`
///
/// # Example
/// ```
/// use l1x_sdk_macros::contract;
///
/// struct Contract {};
///
/// #[contract]
/// impl Contract {
///     pub fn say(msg: String) {
///         // say "hello"
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(input) = syn::parse::<ItemImpl>(item) {
        let struct_type = &input.self_ty;
        let mut generated_code = TokenStream2::new();
        for item in &input.items {
            match item {
                syn::ImplItem::Method(method) => {
                    if !matches!(method.vis, Visibility::Public(_)) {
                        continue;
                    }
                    let ident = &method.sig.ident;
                    let arg_struct = input_struct_deser(&method.sig);
                    let mut arg_list = TokenStream2::new();
                    for arg in &method.sig.inputs {
                        match arg {
                            syn::FnArg::Receiver(_) => todo!(),
                            syn::FnArg::Typed(typed) => {
                                let ident = &typed.pat;
                                arg_list.extend(quote! {
                                    #ident,
                                });
                            }
                        }
                    }
                    let ouput_serialization = match method.sig.output {
                        syn::ReturnType::Default => quote! {},
                        syn::ReturnType::Type(_, _) => quote! {
                            let result = serde_json::to_vec(&result).expect("Failed to serialize the return value using JSON.");
                            l1x_sdk::output(&result);
                        },
                    };
                    generated_code.extend(quote! {
                        #[cfg(target_arch = "wasm32")]
                        #[no_mangle]
                        pub extern "C" fn #ident() {
                            let REENTRANCY_GUARD_KEY: &[u8] = b"__REENTRANCY_GUARD__";
                            let REENTRANCY_GUARD: &[u8] = b"";
                            l1x_sdk::setup_panic_hook();
                            let write_perm = l1x_sdk::storage_write_perm();
                            if write_perm {
                                if l1x_sdk::storage_write(&REENTRANCY_GUARD_KEY, REENTRANCY_GUARD) {
                                    panic!("Found a cross-contract call loop");
                                }
                            } else {
                                if l1x_sdk::storage_read(&REENTRANCY_GUARD_KEY).is_some() {
                                    panic!("Found a cross-contract call loop");
                                }
                            }
                            #arg_struct
                            let Input {
                                #arg_list
                            } = serde_json::from_slice(
                                &l1x_sdk::input().expect("Expected input since method has arguments.")
                            ).expect("Failed to deserialize input from JSON.");
                            let result = #struct_type::#ident(#arg_list);
                            #ouput_serialization
                            if write_perm {
                                l1x_sdk::storage_remove(&REENTRANCY_GUARD_KEY);
                            }
                        }
                    })
                }
                _ => {
                    return TokenStream::from(
                        syn::Error::new(
                            Span::call_site(),
                            "#[contract] only supports methods for now.",
                        )
                        .to_compile_error(),
                    )
                }
            }
        }

        TokenStream::from(quote! {
            #input
            #generated_code
        })
    } else {
        TokenStream::from(
            syn::Error::new(
                Span::call_site(),
                "#[contract] can only be used on impl sections.",
            )
            .to_compile_error(),
        )
    }
}
