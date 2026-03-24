use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Visibility};

/// Derives builder-pattern setter methods for all public fields of a struct.
///
/// Only `pub` fields get setters — private fields are skipped automatically.
///
/// # Field attributes
///
/// Use `#[composite(...)]` on individual fields to customize generation:
///
/// | Option                          | Effect                                                        |
/// |---------------------------------|---------------------------------------------------------------|
/// | `rename = "name"`               | Setter uses `name` instead of the field name                  |
/// | `skip`                          | No setter generated                                           |
/// | `into`                          | Parameter becomes `impl Into<FieldType>`, calls `.into()`     |
/// | `with_types = "impl Into<T>"`   | Explicit parameter type, calls `.into()`                      |
///
/// # Example
///
/// ```ignore
/// #[derive(Default, CompositeWidget)]
/// pub struct CardOptions {
///     pub width: u32,
///     pub height: u32,
///     #[composite(into)]
///     pub title: String,
///     #[composite(skip)]
///     pub on_click: Box<dyn FnMut()>,
///     #[composite(rename = "bg")]
///     pub background_color: Color,
/// }
///
/// // Generated:
/// // impl CardOptions {
/// //     pub fn width(mut self, width: u32) -> Self { self.width = width; self }
/// //     pub fn height(mut self, height: u32) -> Self { self.height = height; self }
/// //     pub fn title(mut self, title: impl Into<String>) -> Self { self.title = title.into(); self }
/// //     // on_click skipped
/// //     pub fn bg(mut self, background_color: Color) -> Self { self.background_color = background_color; self }
/// // }
/// ```
#[proc_macro_derive(CompositeWidget, attributes(composite))]
pub fn derive_composite_widget(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "CompositeWidget only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "CompositeWidget can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut setters = Vec::new();

    for field in fields {
        // Skip non-pub fields
        if !matches!(&field.vis, Visibility::Public(_)) {
            continue;
        }

        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Parse #[composite(...)] attributes
        let attrs = match parse_composite_attrs(field) {
            Ok(attrs) => attrs,
            Err(err) => return err.to_compile_error().into(),
        };

        if attrs.skip {
            continue;
        }

        let setter_name = match &attrs.rename {
            Some(name) => format_ident!("{}", name),
            None => field_name.clone(),
        };

        let setter = if let Some(ref custom_type) = attrs.with_types {
            // with_types = "SomeType" → use that type, call .into()
            let param_type: proc_macro2::TokenStream = custom_type.parse().unwrap_or_else(|_| {
                syn::Error::new_spanned(field, format!("invalid type in with_types: {custom_type}"))
                    .to_compile_error()
            });
            quote! {
                pub fn #setter_name(mut self, #field_name: #param_type) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        } else if attrs.into {
            // into → impl Into<FieldType>, call .into()
            quote! {
                pub fn #setter_name(mut self, #field_name: impl Into<#field_type>) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        } else {
            // Default → direct assignment
            quote! {
                pub fn #setter_name(mut self, #field_name: #field_type) -> Self {
                    self.#field_name = #field_name;
                    self
                }
            }
        };

        setters.push(setter);
    }

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#setters)*
        }
    };

    expanded.into()
}

struct CompositeAttrs {
    rename: Option<String>,
    skip: bool,
    into: bool,
    with_types: Option<String>,
}

fn parse_composite_attrs(field: &syn::Field) -> syn::Result<CompositeAttrs> {
    let mut attrs = CompositeAttrs {
        rename: None,
        skip: false,
        into: false,
        with_types: None,
    };

    for attr in &field.attrs {
        if !attr.path().is_ident("composite") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                attrs.skip = true;
                Ok(())
            } else if meta.path.is_ident("into") {
                attrs.into = true;
                Ok(())
            } else if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.rename = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("with_types") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.with_types = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error(format!(
                    "unknown composite attribute `{}`; expected `rename`, `skip`, `into`, or `with_types`",
                    meta.path.get_ident().map_or("?".into(), |i| i.to_string())
                )))
            }
        })?;
    }

    if attrs.into && attrs.with_types.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "cannot use both `into` and `with_types` on the same field",
        ));
    }

    Ok(attrs)
}
