//! Procedural derive macros for the radiate workspace.
//!
//! Currently exports `#[derive(Freeze)]`, which generates a `Freezable` impl
//! that builds a `Frozen` from the struct's fields. Field-level attributes
//! mirror the serde style:
//!
//! ```ignore
//! #[derive(Freeze)]
//! pub struct PolynomialMutator {
//!     #[freeze(nested)]                  // emit `self.field.freeze()` instead of clone
//!     rate: Rate,
//!     eta: f32,
//!     #[freeze(rename = "contiguity")]   // override the on-wire name
//!     contiguty: f32,
//!     #[freeze(skip)]                    // exclude from the freeze
//!     cache: Mutex<Vec<f32>>,
//!     #[freeze(with = "render_pretty")]  // run `render_pretty(&self.field)` and use the result
//!     encoding: ComplexThing,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta, Path, Token,
    parse_macro_input, punctuated::Punctuated, spanned::Spanned,
};

/// Derive `Freezable` for a struct, building a `Frozen` from its fields.
///
/// See the crate-level docs for supported field attributes.
#[proc_macro_derive(Freeze, attributes(freeze))]
pub fn derive_freeze(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_freeze(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_freeze(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "Freeze can only be derived for structs",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "Freeze requires named fields",
        ));
    };

    let name = &input.ident;
    let (impl_g, ty_g, where_c) = input.generics.split_for_impl();

    let mut withs = Vec::new();
    for field in &named.named {
        let attrs = parse_freeze_attrs(&field.attrs)?;
        if attrs.skip {
            continue;
        }
        let ident = field.ident.as_ref().unwrap();
        let key = attrs.rename.unwrap_or_else(|| ident.to_string());
        let value_expr = if attrs.nested {
            quote! { self.#ident.as_frozen() }
        } else if let Some(func) = attrs.with {
            quote! { (#func)(&self.#ident) }
        } else {
            quote! { self.#ident.clone() }
        };
        withs.push(quote! { .with(#key, #value_expr) });
    }

    let krate = resolve_crate_path();

    Ok(quote! {
        impl #impl_g #krate::Freezable for #name #ty_g #where_c {
            fn as_frozen(&self) -> #krate::Frozen {
                #krate::Frozen::typed::<Self>()
                    #(#withs)*
            }
        }
    })
}

/// Resolve the path the generated code should use to refer to `Freezable`,
/// `Frozen`, etc. Tries `radiate-core` (the canonical home), then `radiate`
/// (the facade re-exports both), then falls back to `::radiate_core` so the
/// error message points at the right place.
fn resolve_crate_path() -> TokenStream2 {
    if let Ok(found) = crate_name("radiate-core") {
        return found_to_path(found);
    }
    if let Ok(found) = crate_name("radiate") {
        return found_to_path(found);
    }
    quote! { ::radiate_core }
}

fn found_to_path(found: FoundCrate) -> TokenStream2 {
    match found {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

#[derive(Default)]
struct FreezeAttrs {
    skip: bool,
    nested: bool,
    rename: Option<String>,
    with: Option<Path>,
}

fn parse_freeze_attrs(attrs: &[Attribute]) -> syn::Result<FreezeAttrs> {
    let mut out = FreezeAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("freeze") {
            continue;
        }
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                Meta::Path(p) if p.is_ident("skip") => out.skip = true,
                Meta::Path(p) if p.is_ident("nested") => out.nested = true,
                Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                    out.rename = Some(expect_string(&nv.value, "rename")?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("with") => {
                    let path_str = expect_string(&nv.value, "with")?;
                    out.with = Some(syn::parse_str::<Path>(&path_str)?);
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "unknown #[freeze(...)] attribute; expected one of: skip, nested, rename = \"...\", with = \"...\"",
                    ));
                }
            }
        }
    }
    Ok(out)
}

fn expect_string(expr: &Expr, attr_name: &str) -> syn::Result<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Ok(s.value()),
        other => Err(syn::Error::new(
            other.span(),
            format!("`{attr_name}` expects a string literal"),
        )),
    }
}
