extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive `FixedLayout` for a type
#[proc_macro_derive(FixedLayout)]
pub fn derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    expand(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = &input.ident;
    let attrs = &input.attrs;
    let ensure = match &input.data {
        syn::Data::Struct(s) => ensure_struct_has_stable_layout(type_name, s, attrs)?,
        _ => todo!(),
    };

    let stream = quote! {
        #ensure
        unsafe impl ::mem_markers::FixedLayout for #type_name {}
    };
    Ok(stream)
}

fn ensure_struct_has_stable_layout(
    type_name: &syn::Ident,
    s: &syn::DataStruct,
    attrs: &Vec<syn::Attribute>,
) -> syn::Result<proc_macro2::TokenStream> {
    let field_types = field_types(s);
    if field_types.is_empty() {
        return Ok(quote! {});
    }

    if !has_stable_repr(attrs) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "FixedLayout trait requires #[repr(C)] or #[repr(transparent)] on structs with fields",
        ));
    }

    let ensure = quote! {
        #(
            ::mem_markers::ensure_fixed_layout::<#field_types>()
        )*
    };
    let ensure_method_name = quote::format_ident!("__ensure_fixed_layout_for_{}", type_name);

    let stream = quote! {
        #[allow(missing_docs)]
        #[allow(non_snake_case)]
        fn #ensure_method_name() {
            #ensure
        }
    };
    Ok(stream)
}

fn field_types(s: &syn::DataStruct) -> Vec<&syn::Type> {
    match &s.fields {
        syn::Fields::Named(n) => n.named.iter().map(|f| &f.ty).collect(),
        syn::Fields::Unnamed(n) => n.unnamed.iter().map(|f| &f.ty).collect(),
        syn::Fields::Unit => vec![],
    }
}

fn has_stable_repr(attrs: &Vec<syn::Attribute>) -> bool {
    for attr in attrs {
        if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("repr") && meta.nested.len() == 1 {
                if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = &meta.nested[0] {
                    if path.is_ident("C") || path.is_ident("transparent") {
                        return true;
                    }
                }
            }
        }
    }
    false
}
