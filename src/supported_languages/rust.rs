use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

use crate::helpers::tree_sitter::{
    find_all_of_kind, find_first_of_kind_with_field_value, node_value,
};
use crate::supported_languages::supported_language::{Language, SupportedLanguage};

pub struct RustAnalyzer {
    inner: Language,
}

impl Display for RustAnalyzer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self {
            inner: Language::Rust,
        }
    }
}

impl SupportedLanguage for RustAnalyzer {
    fn find_correct_node<'a>(
        &self,
        source_file: &str,
        root_tree: &'a Tree,
        parent_identifier: &Option<String>,
        function_identifier: &Option<String>,
    ) -> Result<Node<'a>, &'a str> {
        let cursor = root_tree.walk();
        if let Some(function) = function_identifier {
            let mut candidate_subtrees = vec![];
            if let Some(parent) = parent_identifier {
                let all_impls = find_all_of_kind(cursor, "impl_item");
                for parent_impl in all_impls.into_iter() {
                    let impl_type_node = parent_impl.child_by_field_name("type").unwrap();
                    let impl_type_name = if impl_type_node.kind() == "generic_type" {
                        let type_name_node = impl_type_node.child_by_field_name("type").unwrap();
                        node_value(source_file, type_name_node)
                    } else {
                        node_value(source_file, impl_type_node)
                    };

                    if impl_type_name == parent {
                        candidate_subtrees.push(parent_impl.walk())
                    }
                }

                if candidate_subtrees.is_empty() {
                    return Err("impl block not found");
                }
            } else {
                candidate_subtrees.push(cursor)
            };

            return if let Some(function_node) = candidate_subtrees
                .into_iter()
                .find_map(|cursor| {
                    find_first_of_kind_with_field_value(
                        source_file,
                        cursor,
                        "function_item",
                        "name",
                        function,
                    )
                }) {
                Ok(function_node)
            } else {
                Err("function not found")
            };
        }

        Ok(root_tree.root_node())
    }

    fn language(&self) -> tree_sitter::Language {
        return tree_sitter_rust::language();
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Tree;

    use crate::helpers::tree_sitter::{node_value, parse_source_with_language};
    use crate::supported_languages::rust::RustAnalyzer;
    use crate::supported_languages::supported_language::SupportedLanguage;

    const RUST_SOURCE: &str = r#"// This is a top-level function
fn greet() {
    println!("Hello from top-level function!");
}

// This is a struct
struct Greeter {
    name: String,
}

impl Greeter {
    // This is a method in the struct with the same name as the top-level function
    fn greet(&self) {
        println!("Hello from {}, inside the Greeter struct!", self.name);
    }
}

// A struct with a method named in the same way as the top-level function
struct GenericGreeter<T> {
    name: T,
}

impl<T: std::fmt::Display> GenericGreeter<T> {
    // Method in the struct
    fn greet(&self) {
        println!("Hello from {}, inside the GenericGreeter struct!", self.name);
    }
}

fn main() {
    // Call the top-level function
    greet();

    // Create an instance of the struct and call its method
    let greeter = Greeter {
        name: String::from("Alice"),
    };
    greeter.greet();
}"#;

    fn rust_source_tree() -> Tree {
        parse_source_with_language(RUST_SOURCE, tree_sitter_rust::language())
    }

    #[test]
    fn no_function() {
        let tree = rust_source_tree();
        let ra: RustAnalyzer = Default::default();

        assert_eq!(
            RUST_SOURCE,
            node_value(
                RUST_SOURCE,
                ra.find_correct_node(RUST_SOURCE, &tree, &None, &None)
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn top_level_function() {
        let target = r#"fn greet() {
    println!("Hello from top-level function!");
}"#;

        let tree = rust_source_tree();
        let ra: RustAnalyzer = Default::default();

        assert_eq!(
            target,
            node_value(
                RUST_SOURCE,
                ra.find_correct_node(RUST_SOURCE, &tree, &None, &Some("greet".to_string()))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn method() {
        let target = r#"fn greet(&self) {
        println!("Hello from {}, inside the Greeter struct!", self.name);
    }"#;

        let tree = rust_source_tree();
        let ra: RustAnalyzer = Default::default();

        assert_eq!(
            target,
            node_value(
                RUST_SOURCE,
                ra.find_correct_node(
                    RUST_SOURCE,
                    &tree,
                    &Some("Greeter".to_string()),
                    &Some("greet".to_string()),
                )
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn method_under_parent_with_generics() {
        let target = r#"fn greet(&self) {
        println!("Hello from {}, inside the GenericGreeter struct!", self.name);
    }"#;

        let tree = rust_source_tree();
        let ra: RustAnalyzer = Default::default();

        assert_eq!(
            target,
            node_value(
                RUST_SOURCE,
                ra.find_correct_node(
                    RUST_SOURCE,
                    &tree,
                    &Some("GenericGreeter".to_string()),
                    &Some("greet".to_string()),
                )
                    .unwrap(),
            ),
        )
    }
}
