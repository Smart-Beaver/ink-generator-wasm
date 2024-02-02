use std::borrow::BorrowMut;
use std::error::Error;

use syn::{Attribute, Expr, Field, Fields, File, Ident, ImplItemFn, Item, ItemImpl, ItemMod, ItemStruct, ItemUse, parse_quote, Stmt, UseTree};
use syn::__private::ToTokens;
use syn::spanned::Spanned;

use crate::{Metadata, Standard};
use crate::generator::ast::{extract_attribute_expression, extract_fn_by_ident, extract_fn_implementation_by_attr, extract_fn_implementations, extract_impl_blocks, extract_impl_by_ident, field_to_fn_arg, find_attribute, find_struct_by_attr, generate_field_value, get_ident_from_impl_block, merge_fn_with_start_index, parse_expr_as_number};
use crate::generator::merge::constructor_values::produce_metadata_field_expr;
use crate::generator::merge::uses::{extract_uses, has_use};
use crate::generator::source_parser::{ExtensionContext, ExtensionKind};
use crate::logger::console_log;
use crate::logger::log;

mod uses;
mod errors;
mod constructor_values;

const DEFAULT_LINE_NUMBER_VALUE: usize = 0;

pub trait AstMerger {
    fn merge(base_contract: &File, extensions: Vec<ExtensionContext>, standard: Standard, metadata_common: &Option<Metadata>, single_file_mode: bool) -> Result<File, Box<dyn Error>>;
}

pub type FnChangesCount = u32;

pub struct Merger;


enum FnMergeStrategy {
    Append { line: usize },
    Replace,
}


fn parse_main_mod(root: &mut File, attr: Attribute) -> &mut ItemMod {
    match root.items.iter_mut().find_map(|i| {
        if let Item::Mod(mod_value) = i {
            let contains = mod_value.attrs.iter().any(|root_attr| {
                //Compare just the path segment. Bracket tokens are not important here ;)
                root_attr.path().eq(attr.path())
            });

            if contains {
                return Some(mod_value);
            }
        }
        None
    }) {
        Some(e) => e,
        None => panic!("Main mod not found")
    }
}

fn merge_imports(base_contract: &mut ItemMod, extension: &ItemMod) {
    // Moving uses from extension to main contract
    let all_children = extension.content.as_ref().unwrap().1.to_vec();
    let uses: Vec<&ItemUse> = extract_uses(&all_children);

    if let Some(content) = base_contract.content.as_mut() {
        for single_use in uses {
            if !has_use(&content.1.to_vec(), single_use.clone()).unwrap() {
                let token_stream = single_use.to_token_stream();
                let item = syn::parse2::<Item>(token_stream).unwrap();
                content.1.insert(0, item);
            }
        }
    }
}

fn export_fields_from_struct(input_struct: &ItemStruct) -> Vec<Field> {
    input_struct.fields.iter().cloned().collect()
}

fn strip_field_attributes(field: &Field) -> Field {
    let mut new_field = field.clone();
    new_field.attrs = vec![];
    new_field
}

fn append_fields_to_struct(target_struct: &mut ItemStruct, fields: &[Field]) {
    match target_struct.fields.borrow_mut() {
        Fields::Named(named_fields) => {
            fields.iter().cloned().for_each(|field| {
                named_fields.named.push(strip_field_attributes(&field));
            });
        }
        _ => console_log!("Only named fields are supported, ignoring")
    }
}

fn append_params_to_fn(impl_fn: &mut ImplItemFn, appended_fields: &[Field], search_attr: &Attribute) {
    for field in appended_fields {

        //If field contains expression inside attribute ignore it
        if extract_attribute_expression(&field.attrs, search_attr).is_some() {
            continue;
        }

        let fn_arg_opt = field_to_fn_arg(field);
        match fn_arg_opt {
            None => console_log!("Field has no identifier, ignoring"),
            Some(fn_arg) => {
                console_log!("Appending params to: {}", impl_fn.sig.ident);
                impl_fn.sig.inputs.push(fn_arg);
            }
        }
    }
}


fn extend_constructor_body(
    impl_fn: &mut ImplItemFn,
    appended_fields: &[Field],
    search_attr: &Attribute,
    extension_kind: &ExtensionKind,
    standard: Standard,
    metadata_common: &Option<Metadata>,
) {
    let self_block_opt = impl_fn.block.stmts.iter_mut().find(|stmt| matches!(stmt, Stmt::Expr(_, _)));
    match self_block_opt {
        None => console_log!("No self block found in constructor"),
        Some(self_block) => {
            if let Stmt::Expr(Expr::Struct(inner_struct), _) = self_block {
                for field in appended_fields {
                    let new_field_ident = field.ident.as_ref().unwrap();
                    let custom_expression_opt = extract_attribute_expression(&field.attrs, search_attr);

                    console_log!("Pushing field {:#?} to constructor", new_field_ident);

                    let custom_expression_opt = match extension_kind {
                        ExtensionKind::Metadata => {
                            //@TODO refactor to strategy - eliminate unnecessary match statements
                            Some(produce_metadata_field_expr(extension_kind, standard, field, metadata_common))
                        }
                        _ => {
                            custom_expression_opt//Leave unchanged
                        }
                    };

                    inner_struct.fields.push(generate_field_value(new_field_ident, custom_expression_opt));
                }
            }
        }
    }
}

///Iterate over all implementation blocks and search for function tagged with `#[ink(constructor)]` attribute
fn extend_constructor(
    root_mod: &mut ItemMod,
    appended_fields: &[Field],
    extension_kind: &ExtensionKind,
    standard: Standard,
    metadata_common: &Option<Metadata>,
) {
    //find base contract constructor
    //add fields as parameters to constructor

    match extract_impl_blocks(root_mod) {
        None => console_log!("No impl blocks found in base contract"),
        Some(impl_blocks) => {
            for impl_block in impl_blocks {
                match extract_fn_implementation_by_attr(impl_block, parse_quote!(#[ink(constructor)])) {
                    None => console_log!("No constructor found in base contract"),
                    Some(fn_item) => {
                        let init_attribute: Attribute = parse_quote!(#[smart_beaver::init]);

                        match extension_kind {
                            ExtensionKind::Metadata => {}//pass
                            _ => {
                                //Any other extension
                                append_params_to_fn(fn_item, appended_fields, &init_attribute);
                            }
                        }
                        extend_constructor_body(fn_item, appended_fields, &init_attribute, extension_kind, standard, metadata_common);
                    }
                }
            }
        }
    };
}


fn merge_state_and_constructor(
    root_mod: &mut ItemMod,
    extension: &mut ItemMod,
    extension_kind: &ExtensionKind,
    standard: Standard,
    metadata_common: &Option<Metadata>,
) {
    let target_storage_opt = find_struct_by_attr(root_mod, parse_quote!(#[ink(storage)]));
    let extension_storage_opt = find_struct_by_attr(extension, parse_quote!(#[smart_beaver::storage]));

    match (target_storage_opt, extension_storage_opt) {
        //Both values are present
        (Some(target_storage), Some(extension_storage)) => {
            let extension_fields = export_fields_from_struct(extension_storage);
            console_log!("Extension fields: {:#?}", extension_fields.iter().map(|x| x.ident.clone()).collect::<Vec<_>>());

            append_fields_to_struct(target_storage, &extension_fields);

            extend_constructor(root_mod, &extension_fields, extension_kind, standard, metadata_common);
        }
        _ => console_log!("No storage struct found")
    }
}

///Parses attributes list and tries to determine merge strategy
/// "append" as well as "replace" has optional parameter "line" which is defaulted to 0
/// Having both, "append" and "replace" is invalid
fn parse_fn_merge_strategy(attributes: &[Attribute]) -> Option<FnMergeStrategy> {
    let append_attribute: Attribute = parse_quote!(#[smart_beaver::append]);
    let replace_attribute: Attribute = parse_quote!(#[smart_beaver::replace]);

    let append_attr_opt = find_attribute(attributes, &append_attribute);
    let replace_attr_opt = find_attribute(attributes, &replace_attribute);

    let parse_line_number = |target_attribute: &Attribute| {
        match extract_attribute_expression(attributes, target_attribute) {
            None => DEFAULT_LINE_NUMBER_VALUE,
            Some(expr) => parse_expr_as_number(&expr).unwrap_or(DEFAULT_LINE_NUMBER_VALUE)
        }
    };

    match (append_attr_opt, replace_attr_opt) {
        (Some(append_attr), None) => {
            Some(FnMergeStrategy::Append { line: parse_line_number(append_attr) })
        }
        (None, Some(_)) => {
            Some(FnMergeStrategy::Replace)
        }
        (None, None) => None,
        (Some(_), Some(_)) => None
    }
}

fn merge_functions(target_impl_block: &mut ItemImpl, extension_impl_block: &mut ItemImpl) -> FnChangesCount {
    //Track number of changes made to the target_impl_block
    let mut changes_count: FnChangesCount = 0;

    //read all functions from extension
    extract_fn_implementations(extension_impl_block).iter().for_each(|impl_item| {
        //read function from base contract by its Ident
        let fn_target_opt = extract_fn_by_ident(target_impl_block, &impl_item.sig.ident);

        //Determine merger strategy
        let extension_fn_merge_strategy = parse_fn_merge_strategy(&impl_item.attrs);

        match fn_target_opt {
            None => {
                //Fn doest not exist in target contract
                //If attributes are present, function is ignored
                if extension_fn_merge_strategy.is_none() {
                    console_log!("Copying function: {:#?}", impl_item.sig.ident);
                    changes_count += 1;
                    target_impl_block.items.push(parse_quote!(#impl_item));
                } else {
                    console_log!("Merge strategy not supported for new(not existing on the target module) functions");
                }
            }
            Some(f_target) => {
                //Fn is present in the target contract
                //Strategy selection is required
                match extension_fn_merge_strategy {
                    None => console_log!("No merge strategy found for function: {:#?} - ignoring", f_target.sig.ident),
                    Some(merge_strategy) => {
                        changes_count += 1;
                        match merge_strategy {
                            FnMergeStrategy::Append { line } => {
                                console_log!("Merging function: {:#?}", f_target.sig.ident);
                                merge_fn_with_start_index(f_target, &impl_item.block, line);
                            }
                            FnMergeStrategy::Replace => {
                                console_log!("Overriding function: {:#?}", f_target.sig.ident);
                                f_target.block = impl_item.block.clone();
                            }
                        }
                    }
                }
            }
        }
    });
    changes_count
}

fn merge_impl_blocks(base_contract: &mut ItemMod, extension: &mut ItemMod) {
    let extension_impl_blocks_opt = extract_impl_blocks(extension);
    match extension_impl_blocks_opt {
        None => console_log!("No impl blocks found in extension"),
        Some(impl_blocks) => {
            for extension_impl_block in impl_blocks {
                let ident = get_ident_from_impl_block(extension_impl_block).expect("No ident found, impl block must have a name");

                let impl_target_opt = extract_impl_by_ident(base_contract, ident);

                //Handle merging Impl blocks
                match impl_target_opt {
                    None => {
                        //Impl block is not present in the target contract.
                        console_log!("Copying impl: {:#?}", ident);
                        match base_contract.content.as_mut() {
                            None => console_log!("Base contract does not have content"),
                            Some((_, ref mut content)) => {
                                /*
                                Create copy of the extension impl block and clear it's content.
                                After it, merge functions into the new empty block.
                                This approach eliminated risk of merging not complete function implementations
                                 */

                                let mut extension_impl_block_contentless = extension_impl_block.clone();

                                extension_impl_block_contentless.items.clear();

                                if merge_functions(&mut extension_impl_block_contentless, &mut extension_impl_block.clone()) > 0 {
                                    content.push(parse_quote!(#extension_impl_block_contentless));
                                } else {
                                    console_log!("No functions merged, ignoring impl block: {:?}", ident);
                                }
                            }
                        }
                    }
                    Some(target_item_impl) => {
                        //Function with the same ident exists in the base contract
                        //impl is being overridden
                        //merge functions one by one
                        console_log!("Merging impl: {:#?}", ident);
                        merge_functions(target_item_impl, &mut extension_impl_block.clone());
                    }
                }
            }
        }
    }
}

impl AstMerger for Merger {
    fn merge(base_contract: &File, extensions: Vec<ExtensionContext>, standard: Standard, metadata_common: &Option<Metadata>, single_file_mode: bool) -> Result<File, Box<dyn Error>> {
        let mut common = base_contract.clone();

        filter_global_imports(&mut common, single_file_mode, standard);

        //Search for main mod blocks
        let base_main_mod = parse_main_mod(&mut common, parse_quote! {
            #[ink::contract]
        });

        for extension in extensions {
            let mut ast = extension.ast.clone();
            let ext_main_mod = parse_main_mod(&mut ast, parse_quote! {
                #[smart_beaver::extension]
            });

            merge_imports(base_main_mod, ext_main_mod);

            merge_state_and_constructor(
                base_main_mod,
                ext_main_mod,
                &extension.kind,
                standard,
                metadata_common,
            );

            merge_impl_blocks(base_main_mod, ext_main_mod);
        }

        filter_standard_imports(base_main_mod, single_file_mode, standard);

        console_log!("Merging done");
        Ok(common.clone())
    }
}

///Filter out not needed import statements when single file mode is enabled
/// change required imports to use external crate instead of local files
/// All changes are applied in place to the main mod
/// Removes all mod statements before the main mod with the contract
/// and all use statements
fn filter_global_imports(main_file: &mut File, single_file_mode: bool, standard: Standard) {
    if !single_file_mode {
        return;
    }

    match standard {
        Standard::PSP22 => {
            psp22_filter_global_imports(main_file);
        }
        Standard::PSP34 => {
            //nop
        }
    }
}

fn psp22_filter_global_imports(main_file: &mut File) {
    let contract_attr: Attribute = parse_quote! {#[ink::contract]};

    let filtered_items = main_file.items.iter().filter(|&i| {
        return if let Item::Mod(mod_value) = i {
            mod_value.attrs.iter().any(|root_attr| {
                root_attr.path().eq(contract_attr.path())
            })
        } else {
            false
        };
    }).cloned().collect::<Vec<Item>>();

    main_file.items = filtered_items;
}

///Replace import paths for files from standard
/// `use crate::PSP22` should become `use psp22::PSP22` etc.
fn filter_standard_imports(main_mod: &mut ItemMod, single_file_mode: bool, standard: Standard) {
    if !single_file_mode {
        return;
    }

    match standard {
        Standard::PSP22 => {
            psp22_filter_standard_imports(main_mod, &standard);
        }
        Standard::PSP34 => {
            //nop
        }
    }
}

fn psp22_filter_standard_imports(main_mod: &mut ItemMod, standard: &Standard) {
    if let Some(external_crate) = standard.get_external_crate_name() {
        //Current generator implementation assumes that contract will not have any other special imports
        let main_mod_span = main_mod.span();
        let crate_import_prefix = Ident::new("crate", main_mod_span);

        main_mod.content.iter_mut().for_each(|(_, items)| {
            items.iter_mut().for_each(|item| {
                if let Item::Use(ref mut item_use) = item {
                    if let UseTree::Path(path) = &mut item_use.tree {
                        if path.ident == crate_import_prefix {
                            path.ident = Ident::new(external_crate.name, main_mod_span);
                        }
                    }
                }
            })
        });
    }
}
