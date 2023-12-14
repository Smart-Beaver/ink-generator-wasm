use std::borrow::Borrow;
use std::ops::Deref;

use syn::{Attribute, Block, Expr, FieldValue, FnArg, Ident, ImplItemFn, Item, ItemImpl, ItemMod, ItemStruct, parse_quote, Token, Type};

use crate::logger::console_log;
use crate::logger::log;

//@TODO reduce unnecessary mutable references

pub fn generate_field_value(ident: &Ident, expr: Option<Expr>) -> FieldValue {
    let span = ident.span();

    FieldValue {
        attrs: vec![],
        member: parse_quote!(#ident),
        colon_token: expr.is_some().then(|| Some(Token![:](span))).unwrap_or(None),
        expr: match expr {
            None => parse_quote!(0),
            Some(e) => e
        },
    }
}

pub fn extract_attribute_expression(attrs: &[Attribute], searched_attribute: &Attribute) -> Option<Expr> {
    attrs.iter().find_map(|attr| {
        //Path comparison works only on "smart_beaver::init" part
        if compare_attributes(attr, searched_attribute) {
            return match attr.parse_args::<Expr>() {
                Ok(expr) => Some(expr),
                Err(_) => None
            };
        }
        None
    })
}

pub fn field_to_fn_arg(field: &syn::Field) -> Option<syn::FnArg> {
    field.ident.as_ref().map(|ident| FnArg::Typed(syn::PatType {
        attrs: vec![],
        colon_token: Default::default(),
        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: ident.clone(),
            subpat: None,
        })),
        ty: Box::new(field.ty.clone()),
    }))
}

/// Has to pass in mutable references otherwise rust borrow checker will not allow it
/// Not possible to convert & into &mut:
/// https://doc.rust-lang.org/nomicon/transmutes.html
/// "Transmuting an & to &mut is always Undefined Behavior."
pub fn find_struct_by_attr(root_mod: &mut ItemMod, attr: Attribute) -> Option<&mut ItemStruct> {
    match root_mod.content.as_mut() {
        None => None,
        Some((_brace, items)) => {
            items.iter_mut().find_map(|i| {
                if let Item::Struct(struct_value) = i {
                    if struct_value.attrs.contains(&attr) {
                        return Some(struct_value);
                    }
                }
                None
            })
        }
    }
}

pub fn extract_fn_implementations(impl_block: &mut ItemImpl) -> Vec<&mut ImplItemFn> {
    impl_block.items.iter_mut().filter_map(|item| {
        if let syn::ImplItem::Fn(method) = item {
            return Some(method);
        }
        None
    }).collect()
}

/// Search for first implementation block with given attribute
/// @TODO add param to ignore args expression
pub fn extract_fn_implementation_by_attr(impl_block: &mut ItemImpl, searched_attribute: Attribute) -> Option<&mut ImplItemFn> {
    impl_block.items.iter_mut().find_map(|i| {
        if let syn::ImplItem::Fn(method) = i {
            if method.attrs.iter().any(|a| searched_attribute.eq(a)) {
                return Some(method);
            }
        }
        None
    })
}

/// Check if compare_target is contained in input_fragment
/// compare_target can be only a Path part or entire attribute with nested arguments
fn compare_attributes(input_fragment: &Attribute, compare_target: &Attribute) -> bool {
    match compare_target.parse_args::<Expr>() {
        Ok(_) => {
            //Args present, compare entire attribute
            input_fragment.eq(compare_target)
        }
        Err(_) => {
            //Args not present in searched attribute, so we can compare only paths
            input_fragment.path().eq(compare_target.path())
        }
    }
}

pub fn find_attribute<'a>(attributes: &'a [Attribute], searched_attribute: &Attribute) -> Option<&'a Attribute> {
    return attributes.iter().find(|attr| {
        compare_attributes(attr, searched_attribute)
    });
}

pub fn extract_impl_blocks(root_mod: &mut ItemMod) -> Option<Vec<&mut ItemImpl>> {
    return root_mod.content.as_mut()
        .map(|(_, items)| items)
        .map(|items| {
            items.iter_mut().filter_map(|i| {
                if let Item::Impl(impl_block) = i {
                    return Some(impl_block);
                }
                None
            }).collect::<Vec<&mut ItemImpl>>()
        });
}

//If trait_ is not present then return self_ty
pub fn get_ident_from_impl_block(impl_block: &ItemImpl) -> Option<&Ident> {
    impl_block.trait_.as_ref().map_or_else(|| {
        //trait_ not present, return self type
        match impl_block.self_ty.as_ref() {
            Type::Path(p) => {
                p.path.segments.first().map(|segment| {
                    &segment.ident
                })
            }
            _ => None
        }
    }, |(_, path, _)| {
        path.get_ident()
    })
}

pub fn parse_expr_as_number(attr_expr: &Expr) -> syn::Result<usize> {
    let line_number = match attr_expr {
        Expr::Assign(expr_assign) => {
            match expr_assign.right.deref() {
                Expr::Lit(expr_lit) => {
                    match expr_lit.lit.borrow() {
                        syn::Lit::Int(lit_int) => {
                            lit_int.base10_parse::<usize>()
                        }
                        _ => Err(syn::Error::new_spanned(expr_lit, "Expected integer literal"))
                    }
                }
                _ => Err(syn::Error::new_spanned(expr_assign, "Expected literal"))
            }
        }
        _ => Err(syn::Error::new_spanned(attr_expr, "Expected assignment"))
    };
    line_number
}


pub fn merge_fn_with_start_index(f_target: &mut ImplItemFn, source_block: &Block, start_index: usize) {
    console_log!("Merging function {:?} at index: {:#?}", f_target.sig.ident, start_index);
    //if start index is equal or greater than size of target block - append statements
    let source_stmts = &source_block.stmts;

    if start_index >= f_target.block.stmts.len() {
        //Append at the end
        f_target.block.stmts.extend(source_stmts.clone());
    } else {
        for (idx, stmt) in source_block.stmts.iter().enumerate() {
            f_target.block.stmts.insert(idx + start_index, stmt.clone());
        }
    }
}

pub fn extract_fn_by_ident<'a>(target_impl_block: &'a mut ItemImpl, ident: &'a Ident) -> Option<&'a mut ImplItemFn> {
    let mut output: Option<&mut ImplItemFn> = None;

    for fn_impl in extract_fn_implementations(target_impl_block) {
        if fn_impl.sig.ident.eq(ident) {
            output = Some(fn_impl);
        }
    }

    output
}

pub fn extract_impl_by_ident<'a>(root_mod: &'a mut ItemMod, ident: &'a Ident) -> Option<&'a mut ItemImpl> {
    let mut output: Option<&mut ItemImpl> = None;
    for impl_block in extract_impl_blocks(root_mod).unwrap() {
        let is_trait_eq = impl_block.trait_.as_ref()
            .map(|(_, path, _)| { path.eq(&parse_quote!(#ident)) })
            .unwrap_or(false);

        if (impl_block.trait_.is_none() && impl_block.self_ty.eq(&parse_quote!(#ident))) || is_trait_eq {
            output = Some(impl_block);
        }
    }

    output
}