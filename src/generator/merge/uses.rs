use syn::{Item, ItemUse, UseTree};
use crate::generator::merge::errors::ComparisonError;

fn get_last_path_segments(tree: &UseTree, prefix: &str) -> Result<Vec<String>, ComparisonError> {
    match tree {
        UseTree::Path(path) => {
            let name = path.ident.to_string();
            get_last_path_segments(path.tree.as_ref(), &format!("{prefix}::{name}"))
        },
        UseTree::Name(name) => {
            let name = name.clone().ident;
            Ok(Vec::from([format!("{prefix}::{name}")]))
        },
        UseTree::Rename(_) => Err(ComparisonError::new("Usage of aliases in use clauses is not yet supported. Specify dependencies you want to use!")),
        UseTree::Glob(_) => Err(ComparisonError::new("Usage of * in use clauses is not yet supported. Specify dependencies you want to use!")),
        UseTree::Group(group) => {
            let mut result: Vec<String> = Vec::from([]);
            for item in group.items.iter() {
                let deeper = get_last_path_segments(item, prefix)?;
                for last in deeper.iter() {
                    result.push(last.to_owned().to_string());
                }
            }
            Ok(result)
        }
    }
}

fn already_has_the_same_use(use1: &ItemUse, use2: &ItemUse) -> Result<bool, ComparisonError> {
    for ident1 in get_last_path_segments(&use1.tree, "_")? {
        for ident2 in get_last_path_segments(&use2.tree, "_")? {
            if ident1 == ident2 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}


pub fn has_use(items: &[Item], compare_use: ItemUse) -> Result<bool, ComparisonError> {
    let element = items.iter().find_map(|i| {
        if let Item::Use(item_use) = i {
            let has_same = already_has_the_same_use(item_use, &compare_use);
            if has_same.is_err() {
                return Some(has_same);
            }
            if has_same.unwrap() {
                return Some(Ok(true));
            }
        }
        None
    });

    match element {
        Some(result) => result,
        None => Ok(false),
    }
}

pub fn extract_uses(items: &[Item]) -> Vec<&ItemUse> {
    items.iter().filter_map(|i| {
        if let Item::Use(use_block) = i {
            return Some(use_block);
        }
        None
    }).collect::<Vec<&ItemUse>>()
}
