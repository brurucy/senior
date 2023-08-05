use tree_sitter::{Language, Node, Tree, TreeCursor};
use tree_sitter_traversal::{Order, traverse};

pub fn find_all_of_kind<'a>(cursor: TreeCursor<'a>, kind: &str) -> Vec<Node<'a>> {
    traverse(cursor, Order::Pre)
        .filter(|node| node.kind() == kind)
        .collect()
}

pub fn find_first_of_kind_with_field_value<'a>(
    source: &str,
    cursor: TreeCursor<'a>,
    kind: &str,
    field: &str,
    value: &str,
) -> Option<Node<'a>> {
    traverse(cursor, Order::Pre).find(|node| {
        if node.kind() == kind {
            if let Some(child) = node.child_by_field_name(field) {
                if node_value(source, child) == value {
                    return true;
                }
            }
        }

        false
    })
}

pub fn node_value<'a>(source: &'a str, node: Node<'a>) -> &'a str {
    return node.utf8_text(source.as_bytes()).unwrap();
}

#[allow(dead_code)]
pub fn parse_source_with_language(source: &str, language: Language) -> Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();

    return parser.parse(source.as_bytes(), None).unwrap();
}
