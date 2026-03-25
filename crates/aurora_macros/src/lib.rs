use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemStruct, Visibility};

/// Turns a config struct into a full composite widget.
///
/// Adds an internal storage field, generates builder-pattern setters for
/// each `pub` field, a `new()` constructor, and a complete [`Widget`]
/// implementation that lazily calls [`CompositeBuilder::build`].
///
/// You must also implement [`CompositeBuilder`] for the struct.
///
/// # Field attributes
///
/// Use `#[composite(...)]` on individual fields to customize setters:
///
/// | Option                        | Effect                                                    |
/// |-------------------------------|-----------------------------------------------------------|
/// | `rename = "name"`             | Setter uses `name` instead of the field name              |
/// | `skip`                        | No setter generated                                       |
/// | `into`                        | Parameter becomes `impl Into<FieldType>`, calls `.into()` |
/// | `with_types = "impl Into<T>"` | Explicit parameter type, calls `.into()`                  |
///
/// # Example
///
/// ```ignore
/// #[composite_widget]
/// pub struct MyToggle {
///     pub on_color: Color,
///     pub off_color: Color,
///     #[composite(skip)]
///     pub on_click: Option<Box<dyn FnMut()>>,
/// }
///
/// impl Default for MyToggle {
///     fn default() -> Self {
///         Self {
///             on_color: Color::RED,
///             off_color: Color::GREY,
///             on_click: None,
///             ..  // __composite_inner fills from its field default
///         }
///     }
/// }
///
/// impl CompositeBuilder for MyToggle {
///     fn build(&self) -> Box<dyn Widget> {
///         // compose your widget tree here
///     }
/// }
///
/// // Usage:
/// MyToggle::new().on_color(Color::BLUE)
/// ```
#[proc_macro_attribute]
pub fn composite_widget(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let generics = &input.generics;

    let fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "#[composite_widget] only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    // Collect fields, stripping #[composite(...)] attributes from the output
    let cleaned_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs.retain(|a| !a.path().is_ident("composite"));
            f
        })
        .collect();

    // Also keep references to the originals for attribute parsing
    let original_fields: Vec<_> = fields.iter().collect();

    // Generate setters for pub fields
    let mut setters = Vec::new();
    for field in &original_fields {
        if !matches!(&field.vis, Visibility::Public(_)) {
            continue;
        }

        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let comp_attrs = match parse_composite_attrs(field) {
            Ok(a) => a,
            Err(err) => return err.to_compile_error().into(),
        };

        if comp_attrs.skip {
            continue;
        }

        let setter_name = match &comp_attrs.rename {
            Some(name) => format_ident!("{}", name),
            None => field_name.clone(),
        };

        let setter = if let Some(ref custom_type) = comp_attrs.with_types {
            let param_type: proc_macro2::TokenStream =
                custom_type.parse().unwrap_or_else(|_| {
                    syn::Error::new_spanned(
                        field,
                        format!("invalid type in with_types: {custom_type}"),
                    )
                    .to_compile_error()
                });
            quote! {
                pub fn #setter_name(mut self, #field_name: #param_type) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        } else if comp_attrs.into {
            quote! {
                pub fn #setter_name(mut self, #field_name: impl Into<#field_type>) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        } else {
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
        #(#attrs)*
        #vis struct #struct_name #generics #where_clause {
            #(#cleaned_fields,)*
            /// Internal storage for the built widget tree. Managed by the
            /// `#[composite_widget]` macro — set to `None` in your `Default` impl.
            #[doc(hidden)]
            pub __composite_inner: Option<Box<dyn Widget>>,
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            /// Creates a new composite widget with default configuration.
            pub fn new() -> Self {
                Self::default()
            }

            #(#setters)*
        }

        impl #impl_generics Widget for #struct_name #ty_generics #where_clause {
            fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
                if self.__composite_inner.is_none() {
                    self.__composite_inner = Some(CompositeBuilder::build(self));
                }
                self.__composite_inner.as_mut().unwrap().layout(available, ctx)
            }

            fn paint(&self, canvas: &mut Canvas, rect: Rect) {
                if let Some(ref inner) = self.__composite_inner {
                    inner.paint(canvas, rect);
                }
            }

            fn paint_overlay(&self, canvas: &mut Canvas, rect: Rect) {
                if let Some(ref inner) = self.__composite_inner {
                    inner.paint_overlay(canvas, rect);
                }
            }

            fn children(&self) -> &[Box<dyn Widget>] {
                match &self.__composite_inner {
                    Some(inner) => inner.children(),
                    None => &[],
                }
            }

            fn event(&mut self, event: &WidgetEvent, rect: Rect) -> EventResponse {
                match &mut self.__composite_inner {
                    Some(inner) => inner.event(event, rect),
                    None => EventResponse::default(),
                }
            }

            fn needs_animation(&self) -> bool {
                self.__composite_inner.as_ref().map_or(false, |w| w.needs_animation())
            }
        }
    };

    expanded.into()
}

/// Derives builder-pattern setter methods for all public fields of a struct.
///
/// A simpler alternative to [`composite_widget`] when you only need setters
/// without the automatic [`Widget`] implementation.
///
/// Only `pub` fields get setters — private fields are skipped automatically.
///
/// # Field attributes
///
/// Same as [`composite_widget`]: `rename`, `skip`, `into`, `with_types`.
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
        if !matches!(&field.vis, Visibility::Public(_)) {
            continue;
        }

        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

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
            quote! {
                pub fn #setter_name(mut self, #field_name: impl Into<#field_type>) -> Self {
                    self.#field_name = #field_name.into();
                    self
                }
            }
        } else {
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
