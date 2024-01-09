use syn::{Expr, Field, parse_quote};

use crate::{Metadata, Standard};
use crate::generator::source_parser::ExtensionKind;

//Map extension field to constructor value
pub fn produce_metadata_field_expr(
    extension_kind: &ExtensionKind,
    standard: Standard,
    field: &Field,
    metadata_opt: &Option<Metadata>,
) -> Expr {
    match standard {
        Standard::PSP22 => {
            match &extension_kind {
                ExtensionKind::Metadata => {
                    let name: Expr = metadata_opt.as_ref()
                        .and_then(|m| m.name.clone())
                        .map(|n| parse_quote!(Some(#n)))
                        .unwrap_or(parse_quote!(None));

                    let symbol: Expr = metadata_opt.as_ref()
                        .and_then(|m| m.symbol.clone())
                        .map(|n| parse_quote!(Some(#n)))
                        .unwrap_or(parse_quote!(None));

                    let decimals: Expr = metadata_opt.as_ref()
                        .and_then(|m| m.decimals)
                        .map(|n| parse_quote!(#n))
                        .unwrap_or(parse_quote!(0));

                    match field.ident.as_ref().unwrap().to_string().as_str() {
                        "name" => name,
                        "symbol" => symbol,
                        "decimals" => decimals,
                        _ => parse_quote!(0)
                    }
                }
                _ => parse_quote!(0)
            }
        }
        Standard::PSP34 => {
            //Currently PSP34 does not support metadata extension
            parse_quote!(0)
        }
    }
}