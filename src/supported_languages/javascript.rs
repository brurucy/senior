use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

use crate::helpers::tree_sitter::{
    find_all_of_kind, find_first_of_kind_with_field_value, node_value,
};
use crate::supported_languages::supported_language::{Language, SupportedLanguage};

pub struct JavascriptAnalyser {
    inner: Language,
}

impl Display for JavascriptAnalyser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Default for JavascriptAnalyser {
    fn default() -> Self {
        Self {
            inner: Language::JavaScript,
        }
    }
}

impl SupportedLanguage for JavascriptAnalyser {
    fn find_correct_node<'a>(
        &self,
        source_file: &str,
        root_tree: &'a Tree,
        parent_identifier: Option<&str>,
        function_identifier: Option<&str>,
    ) -> Result<Node<'a>, &'a str> {
        if let Some(function) = function_identifier {
            if let Some(parent) = parent_identifier {
                let all_class_decls = find_all_of_kind(root_tree.walk(), "class_declaration");
                let candidate_correct_class_decl = all_class_decls.into_iter().find(|impl_node| {
                    let class_decl_node = impl_node.child_by_field_name("name").unwrap();

                    node_value(source_file, class_decl_node) == parent
                });

                if let Some(correct_class_decl_node) = candidate_correct_class_decl {
                    let class_body = correct_class_decl_node.child_by_field_name("body").unwrap();
                    let mut cursor = root_tree.walk();

                    return if let Some(function_node) = class_body
                        .children_by_field_name("member", &mut cursor)
                        .filter(|member| member.kind() == "method_definition")
                        .find(|method| {
                            node_value(source_file, method.child_by_field_name("name").unwrap())
                                == function
                        }) {
                        Ok(function_node)
                    } else {
                        Err("method not found")
                    };
                } else {
                    return Err("class not found");
                }
            };

            // If it is a function declaration
            if let Some(function_node) = find_first_of_kind_with_field_value(
                source_file,
                root_tree.walk(),
                "function_declaration",
                "name",
                function,
            ) {
                return Ok(function_node);
            };
            // If it is an arrow function or a expression
            return if let Some(function_node) =
                find_all_of_kind(root_tree.walk(), "lexical_declaration")
                    .into_iter()
                    .filter(|lexical_decl| lexical_decl.child(1).is_some())
                    .find(|lexical_decl| {
                        node_value(
                            source_file,
                            lexical_decl
                                .child(1)
                                .unwrap()
                                .child_by_field_name("name")
                                .unwrap(),
                        ) == function
                    })
            {
                Ok(function_node)
            } else {
                Err("function not found")
            };
        }

        Ok(root_tree.root_node())
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_javascript::language()
    }
}

#[cfg(test)]
mod tests {
    use tree_sitter::Tree;

    use crate::helpers::tree_sitter::{node_value, parse_source_with_language};
    use crate::supported_languages::javascript::JavascriptAnalyser;
    use crate::supported_languages::supported_language::SupportedLanguage;

    // exhausting
    const JAVASCRIPT_SOURCE: &str = r#"// Function Declaration
function greet() {
    console.log("Hello from top-level function declaration!");
}

async function asyncGreet() {
    console.log("Hello from async top-level function declaration!");
}

// Function Expression
const greetExpression = function() {
    console.log("Hello from function expression!");
}

const asyncGreetExpression = async function() {
    console.log("Hello from async function expression!");
}

// Arrow Function
const greetArrow = () => {
    console.log("Hello from arrow function!");
}

const asyncGreetArrow = async () => {
    console.log("Hello from async arrow function!");
}

// Method Definition in a Class
class Greeter {
    greet() {
        console.log("Hello from method in a class!");
    }

    async asyncGreet() {
        console.log("Hello from async method in a class!");
    }
}

// Call all the functions
greet();
asyncGreet();
greetExpression();
asyncGreetExpression();
greetArrow();
asyncGreetArrow();

let greeter = new Greeter();
greeter.greet();
greeter.asyncGreet();
"#;

    fn javascript_source_tree() -> Tree {
        parse_source_with_language(JAVASCRIPT_SOURCE, tree_sitter_javascript::language())
    }

    #[test]
    fn no_function() {
        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            JAVASCRIPT_SOURCE,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, None)
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn top_level_function() {
        let target = r#"function greet() {
    console.log("Hello from top-level function declaration!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("greet"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn async_top_level_function() {
        let target = r#"async function asyncGreet() {
    console.log("Hello from async top-level function declaration!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("asyncGreet"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn top_level_function_expression() {
        let target = r#"const greetExpression = function() {
    console.log("Hello from function expression!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("greetExpression"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn async_top_level_function_expression() {
        let target = r#"const asyncGreetExpression = async function() {
    console.log("Hello from async function expression!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("asyncGreetExpression"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn top_level_arrow_function() {
        let target = r#"const greetArrow = () => {
    console.log("Hello from arrow function!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("greetArrow"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn async_top_level_arrow_function() {
        let target = r#"const asyncGreetArrow = async () => {
    console.log("Hello from async arrow function!");
}"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, None, Some("asyncGreetArrow"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn method() {
        let target = r#"greet() {
        console.log("Hello from method in a class!");
    }"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(JAVASCRIPT_SOURCE, &tree, Some("Greeter"), Some("greet"))
                    .unwrap(),
            ),
        )
    }

    #[test]
    fn async_method() {
        let target = r#"async asyncGreet() {
        console.log("Hello from async method in a class!");
    }"#;

        let tree = javascript_source_tree();
        let ra: JavascriptAnalyser = Default::default();

        assert_eq!(
            target,
            node_value(
                JAVASCRIPT_SOURCE,
                ra.find_correct_node(
                    JAVASCRIPT_SOURCE,
                    &tree,
                    Some("Greeter"),
                    Some("asyncGreet"),
                )
                .unwrap(),
            ),
        )
    }
}
