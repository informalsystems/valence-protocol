use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, Attribute, DataEnum, DeriveInput, GenericArgument,
    PathArguments, Type, TypePath,
};

// Check if theres `skip_update` attribute on the field.
pub(crate) fn has_skip_update_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("skip_update"))
}

pub(crate) fn get_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }
    None
}

// Define a custom struct to parse macro attributes
#[derive(Default)]
struct MacroArgs;

impl Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Ensure the input is empty (no arguments allowed)
        if input.is_empty() {
            Ok(MacroArgs)
        } else {
            Err(syn::Error::new(input.span(), "macro takes no arguments"))
        }
    }
}

// Merges the variants of two enums.
pub(crate) fn merge_variants(
    metadata: TokenStream,
    left: TokenStream,
    right: TokenStream,
) -> TokenStream {
    use syn::Data::Enum;

    // Parse the metadata using our custom parser
    let _args = parse_macro_input!(metadata as MacroArgs);

    let mut left: DeriveInput = parse_macro_input!(left);
    let right: DeriveInput = parse_macro_input!(right);

    if let (
        Enum(DataEnum { variants, .. }),
        Enum(DataEnum {
            variants: to_add, ..
        }),
    ) = (&mut left.data, right.data)
    {
        variants.extend(to_add);

        quote! { #left }.into()
    } else {
        syn::Error::new(left.ident.span(), "variants may only be added for enums")
            .to_compile_error()
            .into()
    }
}
