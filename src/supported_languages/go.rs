use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

use crate::helpers::tree_sitter::{find_all_of_kind, node_value};
use crate::supported_languages::supported_language::{Language, SupportedLanguage};

pub struct GoAnalyser {
    inner: Language,
}

impl Default for GoAnalyser {
    fn default() -> Self {
        Self {
            inner: Language::Go,
        }
    }
}

impl Display for GoAnalyser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl SupportedLanguage for GoAnalyser {
    fn find_correct_node<'a>(
        &self,
        source_file: &str,
        root_tree: &'a Tree,
        parent_identifier: Option<&str>,
        function_identifier: Option<&str>,
    ) -> Result<Node<'a>, &'a str> {
        if let Some(function) = function_identifier {
            return if let Some(parent) = parent_identifier {
                let all_method_decls = find_all_of_kind(root_tree.walk(), "method_declaration");

                let candidate_function_node = all_method_decls.into_iter().find(|method_decl| {
                    let method_name = node_value(
                        source_file,
                        method_decl.child_by_field_name("name").unwrap(),
                    );

                    let receiver_node = method_decl.child_by_field_name("receiver").unwrap();

                    // method receiver nodes only have one argument, a single parameter declaration
                    let parameter_declaration_node = receiver_node.child(1).unwrap();

                    // It always has a type
                    let receiver_type = parameter_declaration_node
                        .child_by_field_name("type")
                        .unwrap();

                    // That can either be a pointer, or not
                    return if receiver_type.kind() == "type_identifier" {
                        node_value(source_file, receiver_type) == parent && method_name == function
                    } else {
                        let pointer_receiver_type = receiver_type.child(1).unwrap();

                        node_value(source_file, pointer_receiver_type) == parent
                            && method_name == function
                    };
                });

                if let Some(function_node) = candidate_function_node {
                    return Ok(function_node);
                }

                Err("method not found")
            } else {
                let all_functions = find_all_of_kind(root_tree.walk(), "function_declaration");

                if let Some(function_node) = all_functions.into_iter().find(|function_decl| {
                    let function_name = node_value(
                        source_file,
                        function_decl.child_by_field_name("name").unwrap(),
                    );

                    function_name == function
                }) {
                    return Ok(function_node);
                }

                Err("function not found")
            };
        };
        Ok(root_tree.root_node())
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_go::language()
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Tree;

    use crate::helpers::tree_sitter::{node_value, parse_source_with_language};
    use crate::supported_languages::go::GoAnalyser;
    use crate::supported_languages::supported_language::SupportedLanguage;

    const GO_SOURCE: &str = r#"package main

import "fmt"

// Top-level function declaration
func greet() {
    fmt.Println("Hello from top-level function!")
}

// A struct with a method named in the same way as the top-level function
type Greeter struct {
    name string
}

// Method in the struct
func (g Greeter) greet() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}

func (g *Greeter) greetPointer() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}

func main() {
    // Call the top-level function
    greet()

    // Create instances of the generic struct and call its method
    greeter := Greeter{name: "Bob"}
    greeter.greet()
}"#;

    fn go_source_tree() -> Tree {
        parse_source_with_language(GO_SOURCE, tree_sitter_go::language())
    }

    #[test]
    fn no_function() {
        let tree = go_source_tree();
        let ra: GoAnalyser = Default::default();

        assert_eq!(
            GO_SOURCE,
            node_value(
                GO_SOURCE,
                ra.find_correct_node(GO_SOURCE, &tree, None, None).unwrap()
            ),
        )
    }

    #[test]
    fn top_level_function() {
        let target = r#"func greet() {
    fmt.Println("Hello from top-level function!")
}"#;

        let tree = go_source_tree();
        let ra: GoAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                GO_SOURCE,
                ra.find_correct_node(GO_SOURCE, &tree, None, Some("greet"))
                    .unwrap()
            ),
        )
    }

    #[test]
    fn method() {
        let target = r#"func (g Greeter) greet() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}"#;

        let tree = go_source_tree();
        let ra: GoAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                GO_SOURCE,
                ra.find_correct_node(GO_SOURCE, &tree, Some("Greeter"), Some("greet"))
                    .unwrap()
            ),
        )
    }

    #[test]
    fn pointer_method() {
        let target = r#"func (g *Greeter) greetPointer() {
    fmt.Printf("Hello from %s, inside the Greeter struct!\n", g.name)
}"#;

        let tree = go_source_tree();
        let ra: GoAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                GO_SOURCE,
                ra.find_correct_node(GO_SOURCE, &tree, Some("Greeter"), Some("greetPointer"))
                    .unwrap()
            ),
        )
    }
}
