use crate::names::{sanitize_field_name, to_pascal_case};
use crate::tree::TreeNode;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;

/// Generate all struct definitions from the key tree.
///
/// Returns a `Vec` of `TokenStream`s, each defining one struct.
/// Struct names are derived from the path: root is `Lang`, children are
/// `Lang` + PascalCase(key segments). All structs derive `Debug, Clone, Copy`.
pub fn generate_struct_defs(
    tree: &BTreeMap<String, TreeNode>,
    parent_name: &str,
) -> Vec<TokenStream> {
    let mut structs = Vec::new();
    generate_struct_recursive(tree, parent_name, &mut structs);
    structs
}

fn generate_struct_recursive(
    tree: &BTreeMap<String, TreeNode>,
    struct_name: &str,
    out: &mut Vec<TokenStream>,
) {
    let struct_ident = format_ident!("{}", struct_name);
    let mut fields = Vec::new();

    for (key, node) in tree {
        let field_ident = format_ident!("{}", sanitize_field_name(key));
        match node {
            TreeNode::Leaf => {
                fields.push(quote! {
                    pub #field_ident: &'static str
                });
            }
            TreeNode::Branch(children) => {
                let child_struct_name = format!("{}{}", struct_name, to_pascal_case(key));
                let child_struct_ident = format_ident!("{}", &child_struct_name);
                fields.push(quote! {
                    pub #field_ident: #child_struct_ident
                });
                generate_struct_recursive(children, &child_struct_name, out);
            }
        }
    }

    out.push(quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct #struct_ident {
            #(#fields),*
        }
    });
}

/// Generate a const expression that constructs the root struct for one locale.
///
/// `values` is the flat map of dotted keys → string values for this locale.
/// `tree` is the key tree, and `parent_struct_name` is the struct name.
pub fn generate_const_expr(
    tree: &BTreeMap<String, TreeNode>,
    values: &BTreeMap<String, String>,
    parent_struct_name: &str,
) -> TokenStream {
    generate_const_recursive(tree, values, parent_struct_name, "")
}

fn generate_const_recursive(
    tree: &BTreeMap<String, TreeNode>,
    values: &BTreeMap<String, String>,
    struct_name: &str,
    prefix: &str,
) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);
    let mut field_inits = Vec::new();

    for (key, node) in tree {
        let field_ident = format_ident!("{}", sanitize_field_name(key));
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        match node {
            TreeNode::Leaf => {
                let val = values.get(&full_key).cloned().unwrap_or_default();
                field_inits.push(quote! { #field_ident: #val });
            }
            TreeNode::Branch(children) => {
                let child_struct_name = format!("{}{}", struct_name, to_pascal_case(key));
                let child_expr =
                    generate_const_recursive(children, values, &child_struct_name, &full_key);
                field_inits.push(quote! { #field_ident: #child_expr });
            }
        }
    }

    quote! {
        #struct_ident {
            #(#field_inits),*
        }
    }
}
