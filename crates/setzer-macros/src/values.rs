//! Front-matter values, their shapes, and the code emitted for them.

use crate::markdown::{emit_parts, PartSpec};
use proc_macro2::{Ident, Span, TokenStream as Ts};
use quote::{format_ident, quote};
use std::collections::BTreeMap;

pub type Values = BTreeMap<String, Value>;

#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    StrList(Vec<String>),
    Table(Values),
    TableList(Vec<Values>),
    Body(Vec<PartSpec>),
}

pub fn from_table(t: &toml::Table) -> Result<Values, String> {
    let mut out = Values::new();
    for (k, v) in t {
        let value = match v {
            toml::Value::String(s) => Value::Str(s.clone()),
            toml::Value::Integer(i) => Value::Int(*i),
            toml::Value::Float(f) => Value::Float(*f),
            toml::Value::Boolean(b) => Value::Bool(*b),
            toml::Value::Datetime(d) => Value::Str(d.to_string()),
            toml::Value::Table(inner) => Value::Table(from_table(inner)?),
            toml::Value::Array(items) => {
                if items.iter().all(|i| i.is_str()) {
                    Value::StrList(
                        items
                            .iter()
                            .map(|i| i.as_str().unwrap().to_string())
                            .collect(),
                    )
                } else if items.iter().all(|i| i.is_table()) {
                    Value::TableList(
                        items
                            .iter()
                            .map(|i| from_table(i.as_table().unwrap()))
                            .collect::<Result<_, _>>()?,
                    )
                } else {
                    return Err(format!(
                        "key `{k}`: arrays must be all strings or all tables"
                    ));
                }
            }
        };
        out.insert(k.clone(), value);
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Str,
    Int,
    Float,
    Bool,
    StrList,
    Body,
    Table(BTreeMap<String, Shape>),
    TableList(BTreeMap<String, Shape>),
}

impl Shape {
    pub fn of(values: &Values) -> Result<Shape, String> {
        let mut fields = BTreeMap::new();
        for (k, v) in values {
            fields.insert(k.clone(), Shape::of_value(k, v)?);
        }
        Ok(Shape::Table(fields))
    }

    fn of_value(key: &str, v: &Value) -> Result<Shape, String> {
        Ok(match v {
            Value::Str(_) => Shape::Str,
            Value::Int(_) => Shape::Int,
            Value::Float(_) => Shape::Float,
            Value::Bool(_) => Shape::Bool,
            Value::StrList(_) => Shape::StrList,
            Value::Body(_) => Shape::Body,
            Value::Table(t) => Shape::of(t)?,
            Value::TableList(items) => {
                let first = items
                    .first()
                    .ok_or(format!("key `{key}`: empty array of tables has no shape"))?;
                let Shape::Table(shape) = Shape::of(first)? else {
                    unreachable!()
                };
                for item in &items[1..] {
                    let Shape::Table(other) = Shape::of(item)? else {
                        unreachable!()
                    };
                    if other != shape {
                        return Err(format!(
                            "key `{key}`: every table in the array needs the same keys"
                        ));
                    }
                }
                Shape::TableList(shape)
            }
        })
    }
}

/// First difference between two shapes, phrased for a human.
pub fn diff(a: &Shape, b: &Shape, at: &str) -> Option<String> {
    match (a, b) {
        (Shape::Table(fa), Shape::Table(fb)) | (Shape::TableList(fa), Shape::TableList(fb)) => {
            let prefix = if matches!(a, Shape::TableList(_)) {
                format!("{at}[]")
            } else {
                at.to_string()
            };
            for k in fa.keys() {
                if !fb.contains_key(k) {
                    return Some(format!("is missing the key `{}`", join(&prefix, k)));
                }
            }
            for k in fb.keys() {
                if !fa.contains_key(k) {
                    return Some(format!("has an extra key `{}`", join(&prefix, k)));
                }
            }
            fa.iter()
                .find_map(|(k, sa)| diff(sa, &fb[k], &join(&prefix, k)))
        }
        _ if a == b => None,
        _ => Some(format!("has `{at}` as {b:?}, expected {a:?}")),
    }
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

pub fn emit_struct(name: &Ident, shape: &Shape, out: &mut Vec<Ts>) {
    let (Shape::Table(fields) | Shape::TableList(fields)) = shape else {
        return;
    };
    let mut decls = Vec::new();
    for (key, shape) in fields {
        let field = field_ident(key);
        let ty = match shape {
            Shape::Str => quote!(&'static str),
            Shape::Int => quote!(i64),
            Shape::Float => quote!(f64),
            Shape::Bool => quote!(bool),
            Shape::StrList => quote!(&'static [&'static str]),
            Shape::Body => quote!(::setzer::Body),
            Shape::Table(_) => {
                let sub = sub_type(name, key);
                emit_struct(&sub, shape, out);
                quote!(#sub)
            }
            Shape::TableList(_) => {
                let sub = sub_type(name, key);
                emit_struct(&sub, shape, out);
                quote!(&'static [#sub])
            }
        };
        decls.push(quote!(pub #field: #ty,));
    }
    out.push(quote! { pub struct #name { #(#decls)* } });
}

pub fn emit_value(ty: &Ident, v: &Value) -> Ts {
    match v {
        Value::Str(s) => quote!(#s),
        Value::Int(i) => quote!(#i),
        Value::Float(f) => quote!(#f),
        Value::Bool(b) => quote!(#b),
        Value::StrList(items) => quote!(&[#(#items),*]),
        Value::Body(parts) => emit_parts(parts),
        Value::Table(fields) => {
            let inits = fields.iter().map(|(k, v)| {
                let field = field_ident(k);
                let value = emit_value(&sub_type(ty, k), v);
                quote!(#field: #value,)
            });
            quote!(#ty { #(#inits)* })
        }
        Value::TableList(items) => {
            let elems = items
                .iter()
                .map(|t| emit_value(ty, &Value::Table(t.clone())));
            quote!(&[#(#elems),*])
        }
    }
}

fn field_ident(key: &str) -> Ident {
    let snake = key.replace('-', "_");
    if syn::parse_str::<Ident>(&snake).is_ok() {
        Ident::new(&snake, Span::call_site())
    } else {
        format_ident!("r#{}", snake)
    }
}

fn sub_type(parent: &Ident, key: &str) -> Ident {
    let mut camel = String::new();
    for part in key.split(['_', '-']) {
        let mut c = part.chars();
        if let Some(f) = c.next() {
            camel.push(f.to_ascii_uppercase());
            camel.push_str(c.as_str());
        }
    }
    format_ident!("{parent}{camel}")
}
