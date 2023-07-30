use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

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
        return self.inner.fmt(f);
    }
}

impl SupportedLanguage for GoAnalyser {
    fn find_correct_node<'a>(&self, source_file: &str, root_tree: &'a Tree, parent_identifier: Option<&str>, function_identifier: Option<&str>) -> Result<Node<'a>, &'a str> {
        todo!()
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_go::language()
    }
}