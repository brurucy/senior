use std::env;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;

use clap::Parser;
use colored::*;
use inquire::Confirm;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

use crate::helpers::tree_sitter::node_value;
use crate::supported_languages::supported_language::{detect_language, SupportedLanguage};

mod helpers;
mod supported_languages;

const OPENAI_API_KEY_ENV_VAR_KEY: &str = "OPENAI_API_KEY";
const COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

struct Optimizer {
    supported_language: Box<dyn SupportedLanguage>,
    file_name: String,
    parent_element: Option<String>,
    function_name: Option<String>,
    source_file: String,
    code: String,
    extra_context: Option<String>,
    model: String,
    theme: String,
    dry_run: bool,
    add_comments: bool,
    skip_prompt: bool,
    tree: Option<Tree>,
    function_node_id: usize,
    parser: tree_sitter::Parser,
}

fn draft_instructions(
    code: &str,
    function_name: &Option<String>,
    add_comments: bool,
    extra_context: &Option<String>,
) -> String {
    let add_comments = if add_comments {
        "Do"
    } else {
        "Absolutely do not"
    };
    let task = if let Some(function) = function_name {
        format!(
            "the function named {} contained in the following code:\n {}",
            function, code
        )
    } else {
        format!("the following code:\n {}", code)
    };

    format!(
        r#"You are a senior software engineer.

Your task is to optimise {}

Strictly adhere to the following instructions:
1. Do not change the type signature.
2. Only propose small, incremental changes.
3. {} add comments.
4. Respond only with code, and no commentary or explanations.
5. If the code cannot be optimised further, respond with "OPTIMAL"
{}
"#,
        task,
        add_comments,
        if let Some(context) = extra_context {
            format!("6. {}", context)
        } else {
            "".to_string()
        }
    )
}

impl Optimizer {
    fn new(
        file_name: &str,
        parent_element: Option<String>,
        function_name: Option<String>,
        extra_context: Option<String>,
        theme: &str,
        model: &str,
        dry_run: bool,
        add_comments: bool,
        skip_prompt: bool,
    ) -> Self {
        return Self {
            supported_language: detect_language(file_name).unwrap(),
            file_name: file_name.to_string(),
            code: "".to_string(),
            parent_element,
            function_name,
            source_file: read_to_string(file_name).unwrap(),
            extra_context,
            model: model.to_string(),
            theme: theme.to_string(),
            dry_run,
            add_comments,
            skip_prompt,
            tree: None,
            function_node_id: 0,
            parser: tree_sitter::Parser::new(),
        };
    }
    fn build(&mut self) -> Result<(), String> {
        self.parser = tree_sitter::Parser::new();
        self.parser
            .set_language(self.supported_language.language())
            .unwrap();

        if let Some(tree) = self.parser.parse(&self.source_file, None) {
            self.tree = Some(tree);

            Ok(())
        } else {
            Err(String::from("failed to parse source file"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct OpenAIChatResponseChoice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatResponseChoice>,
}

fn do_render(tree: &Tree, src: &str, editor: &impl tree_sitter_edit::Editor) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    tree_sitter_edit::render(&mut v, tree, src.as_bytes(), editor).unwrap();
    v
}

impl Optimizer {
    async fn optimise(&mut self, secret: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let model = self.model.clone();
        let node = self.supported_language.find_correct_node(
            &self.source_file,
            self.tree.as_ref().unwrap(),
            &self.parent_element,
            &self.function_name,
        );
        match node {
            Ok(function_node) => {
                self.code = node_value(&self.source_file, function_node).to_string();
                self.function_node_id = function_node.id();
            }
            // :D code gore
            Err(e) => {
                return Err(e.to_string());
            }
        }

        let body = OpenAIChatRequest {
            model,
            messages: vec![Message {
                role: "system".to_string(),
                content: draft_instructions(
                    &self.code,
                    &self.function_name,
                    self.add_comments,
                    &self.extra_context,
                ),
            }],
        };

        let url = Url::parse(COMPLETION_URL).unwrap();

        match client.post(url).bearer_auth(secret).json(&body).build() {
            Ok(req) => match client.execute(req).await {
                Ok(resp) => match resp.json::<OpenAIChatResponse>().await {
                    Ok(parsed_resp) => {
                        let content = parsed_resp.choices[0].message.content.clone();

                        Ok(content)
                    }
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
    fn apply_suggestion_to_source_file(&mut self, suggestion: &[u8]) -> String {
        let editor = tree_sitter_edit::Replace {
            id: tree_sitter_edit::NodeId {
                id: self.function_node_id,
            },
            bytes: suggestion.to_vec(),
        };
        let r = do_render(self.tree.as_ref().unwrap(), &self.source_file, &editor);

        String::from_utf8(r).unwrap()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Location of the source code file
    #[arg()]
    file_path: String,

    /// Name of the function or method that will be searched for in the file at the given file_path
    #[arg(short = 'f', long)]
    function_identifier: Option<String>,

    /// Parent syntactic element(Class, struct, ... etc) of the function to be optimised. If not given
    /// then the first function with function_name argument as identifier will be picked up
    #[arg(short = 'p', long)]
    parent_identifier: Option<String>,

    /// The OpenAI model. Check out https://platform.openai.com/docs/models/overview
    #[arg(short, long)]
    model: Option<String>,

    /// Anything else you would like to tell your chosen LLM
    #[arg(short, long)]
    extra_context: Option<String>,

    /// Setting this option to true will print out the suggestion without a confirmation prompt
    #[arg(short, long)]
    dry_run: bool,

    /// Setting this option to true will print and automatically overwrite the function in the source file
    #[arg(short, long)]
    skip_prompt: bool,

    /// Should the new code have comments?
    #[arg(short, long)]
    add_comments: bool,

    /// The `bat` theme. Check out https://github.com/sharkdp/bat/tree/master/assets/themes for a list of available themes
    #[arg(short, long)]
    theme: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let secret = if let Ok(api_key) = env::var(OPENAI_API_KEY_ENV_VAR_KEY) {
        api_key
    } else {
        panic!("{}", "the OPENAI_API_KEY env var is not present".red())
    };

    let mut opt = Optimizer::new(
        &args.file_path,
        args.parent_identifier,
        args.function_identifier,
        args.extra_context,
        &if let Some(theme) = args.theme {
            theme
        } else {
            "Dracula".to_string()
        },
        &if let Some(model) = args.model {
            model
        } else {
            "gpt-3.5-turbo".to_string()
        },
        args.dry_run,
        args.add_comments,
        args.skip_prompt,
    );
    if let Err(e) = opt.build() {
        panic!("{}", e.red())
    }

    match opt.optimise(&secret).await {
        Ok(suggestion) => {
            if &suggestion == "OPTIMAL" {
                println!("{}", "the current code is already optimal.".green())
            } else {
                let supported_language = opt.supported_language.to_string();

                bat::PrettyPrinter::new()
                    .input_from_bytes(suggestion.as_bytes())
                    .language(&supported_language)
                    .vcs_modification_markers(true)
                    .theme(opt.theme.clone())
                    .print()
                    .unwrap();

                if !opt.dry_run {
                    let mut overwrite_file = true;
                    if !opt.skip_prompt {
                        overwrite_file = Confirm::new("Apply suggestion?")
                            .with_default(false)
                            .prompt()
                            .unwrap();
                    }
                    if overwrite_file {
                        let edited_file = opt.apply_suggestion_to_source_file(suggestion.as_bytes());
                        let file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(opt.file_name);
                        match file {
                            Ok(mut opened_file) => {
                                if let Err(e) = opened_file.write(edited_file.as_bytes()) {
                                    panic!("{}", e.to_string().red())
                                }
                            }
                            Err(e) => panic!("{}", e.to_string().red()),
                        }
                    }
                }
            }
        }
        Err(e) => panic!("{}", e.red()),
    }
}
