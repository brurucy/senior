use std::fmt::{Display, Formatter};

use tree_sitter::{Node, Tree};

use Language::*;

use crate::supported_languages::go::GoAnalyser;
use crate::supported_languages::javascript::JavascriptAnalyser;
use crate::supported_languages::rust::RustAnalyzer;
use crate::supported_languages::typescript::TypescriptAnalyser;

pub trait SupportedLanguage: Display {
    fn find_correct_node<'a>(
        &self,
        source_file: &str,
        root_tree: &'a Tree,
        parent_identifier: &Option<String>,
        function_identifier: &Option<String>,
    ) -> Result<Node<'a>, &'a str>;
    fn language(&self) -> tree_sitter::Language;
}

pub enum Language {
    Go,
    JavaScript,
    Rust,
    TypeScript,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stringified_value = match self {
            Go => "go",
            JavaScript => "javascript",
            Rust => "rust",
            TypeScript => "typescript",
        };

        write!(f, "{}", stringified_value)
    }
}

/// File globs that identify languages based on the file path.
pub fn language_globs(language: &Language) -> Vec<glob::Pattern> {
    let glob_strs: &'static [&'static str] = match language {
        Go => &["*.go"],
        JavaScript => &["*.cjs", "*.js", "*.mjs"],
        Rust => &["*.rs"],
        TypeScript => &["*.ts"],
    };

    glob_strs
        .iter()
        .map(|name| glob::Pattern::new(name).unwrap())
        .collect()
}

pub fn detect_language(file_name: &str) -> Result<Box<dyn SupportedLanguage>, &str> {
    let candidate_language = vec![Rust, Go, JavaScript, TypeScript]
        .into_iter()
        .find(|language| {
            language_globs(language)
                .into_iter()
                .any(|patt| patt.matches(file_name))
        });

    if let Some(language) = candidate_language {
        match language {
            Rust => Ok(Box::<RustAnalyzer>::default()),
            Go => Ok(Box::<GoAnalyser>::default()),
            JavaScript => Ok(Box::<JavascriptAnalyser>::default()),
            TypeScript => Ok(Box::<TypescriptAnalyser>::default()),
        }
    } else {
        Err("not a supported file")
    }
}
