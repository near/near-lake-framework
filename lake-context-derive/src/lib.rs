#![doc = include_str!("../README.md")]
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(LakeContext)]
pub fn lake_context_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Build the trait impl.
    // Iterate over all fields and for each field generate a call to `execute_before_run`.
    // if the field is a an impl of LakeContext, then call `execute_before_run` on the struct.

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
        ..
    }) = &input.data
    {
        named
    } else {
        unimplemented!();
    };

    let calls_before_run = fields
        .iter()
        .filter(|f| {
            let ty = &f.ty;
            if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
                if let Some(ident) = path.get_ident() {
                    ident == "LakeContext"
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|f| {
            let name = &f.ident;
            quote! { self.#name.execute_before_run(block); }
        });

    let calls_after_run = fields
        .iter()
        .rev()
        .filter(|f| {
            let ty = &f.ty;
            if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
                if let Some(ident) = path.get_ident() {
                    ident == "LakeContext"
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|f| {
            let name = &f.ident;
            quote! { self.#name.execute_after_run(); }
        });

    let expanded = quote! {
        // The generated impl.
        impl near_lake_framework::LakeContextExt for #name {
            fn execute_before_run(&self, block: &near_lake_primitives::block::Block) {
                #( #calls_before_run )*
            }

            fn execute_after_run(&self) {
                #( #calls_after_run )*
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
