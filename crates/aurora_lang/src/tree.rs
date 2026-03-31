use std::collections::BTreeMap;

/// A node in the locale key tree.
#[derive(Debug)]
pub enum TreeNode {
    /// Terminal key — becomes a `&'static str` field.
    Leaf,
    /// Intermediate key — becomes a nested struct.
    Branch(BTreeMap<String, TreeNode>),
}

/// Build a unified key tree from a flat set of dotted keys.
///
/// All locales must share the same key set (validated before calling this).
pub fn build_key_tree(all_keys: &[String]) -> Result<BTreeMap<String, TreeNode>, String> {
    let mut root: BTreeMap<String, TreeNode> = BTreeMap::new();

    for key in all_keys {
        let parts: Vec<&str> = key.split('.').collect();
        insert_path(&mut root, &parts, key)?;
    }

    Ok(root)
}

fn insert_path(
    node: &mut BTreeMap<String, TreeNode>,
    parts: &[&str],
    full_key: &str,
) -> Result<(), String> {
    if parts.len() == 1 {
        let entry = node.entry(parts[0].to_string()).or_insert(TreeNode::Leaf);
        if let TreeNode::Branch(_) = entry {
            return Err(format!(
                "key conflict: `{}` is used as both a value and a namespace",
                full_key
            ));
        }
        return Ok(());
    }

    let entry = node
        .entry(parts[0].to_string())
        .or_insert_with(|| TreeNode::Branch(BTreeMap::new()));

    match entry {
        TreeNode::Branch(children) => insert_path(children, &parts[1..], full_key),
        TreeNode::Leaf => Err(format!(
            "key conflict: `{}` requires `{}` to be a namespace, but it is already a value",
            full_key, parts[0]
        )),
    }
}
