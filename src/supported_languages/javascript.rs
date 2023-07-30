use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

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
    fn find_correct_node<'a>(&self, source_file: &str, root_tree: &'a Tree, parent_identifier: Option<&str>, function_identifier: Option<&str>) -> Result<Node<'a>, &'a str> {
        todo!()
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_javascript::language()
    }
}