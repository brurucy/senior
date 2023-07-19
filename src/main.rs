use inquire::Confirm;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;

use tree_sitter_traversal::{traverse_tree, Order};

const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
const COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

struct Optimizer<'a> {
    file_name: String,
    function_name: String,
    source_file: String,
    function: String,
    optional_context: String,
    model: String,
    theme: String,
    dry_run: bool,
    add_comments: bool,
    skip_prompt: bool,
    tree: Option<tree_sitter::Tree>,
    function_node: Option<tree_sitter::Node<'a >>,
}

impl<'a> Default for Optimizer<'a> {
    fn default() -> Self {
        return Self {
            file_name: Default::default(),
            function_name: Default::default(),
            source_file: Default::default(),
            function: Default::default(),
            optional_context: Default::default(),
            model: "gpt-3.5-turbo".to_string(),
            theme: Default::default(),
            dry_run: false,
            add_comments: false,
            skip_prompt: false,
            tree: Default::default(),
            function_node: Default::default(),
        };
    }
}

fn draft_instructions(function: &str, add_comments: bool, optional_context: &str) -> String {
    let add_comments = if add_comments { "do" } else { "don't" };

    return format!(
        r#"You are a senior software engineer.

Your task is to optimise the following function: {}

Strictly adhere to the following instructions:
1. Optimise it without changing the type signature, and only propose small, incremental changes
2. {} add comments.
3. Respond only with code, and nothing else
3. If the code cannot be optimised further, respond with "OPTIMAL"
4. {}
"#,
        function, add_comments, optional_context
    );
}

impl<'a> Optimizer<'a> {
    fn new(
        file_name: &str,
        function_name: &str,
        optional_context: &str,
        theme: &str,
        model: &str,
        dry_run: bool,
        add_comments: bool,
        skip_prompt: bool,
    ) -> Self {
        let mut optimizer: Optimizer = Default::default();

        optimizer.file_name = file_name.to_string();
        optimizer.function_name = function_name.to_string();
        optimizer.source_file = read_to_string(&optimizer.file_name).unwrap();
        optimizer.optional_context = optional_context.to_string();
        optimizer.model = model.to_string();
        optimizer.theme = theme.to_string();
        optimizer.dry_run = dry_run;
        optimizer.add_comments = add_comments;
        optimizer.skip_prompt = skip_prompt;
        optimizer.tree = None;

        return optimizer;
    }
    fn build(&mut self) -> Result<(), String> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        if let Some(tree) = parser.parse(&self.source_file, None) {
            let source_bytes = self.source_file.as_bytes();
            if let Some(node) = traverse_tree(&tree, Order::Pre).find(|node| {
                let node_kind = node.kind();
                if node_kind == "function_item" {
                    let node_function_name = node.child_by_field_name("name").unwrap();
                    return node_function_name.utf8_text(&source_bytes).unwrap()
                        == self.function_name;
                }

                return false;
            }) {
                self.function = node.utf8_text(source_bytes).unwrap().to_string();
            } else {
                return Err(format!("failed to find function: {}", self.function_name));
            }
        } else {
            return Err(String::from("failed to parse source file"));
        }

        return Ok(());
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

impl<'a> Optimizer<'a> {
    async fn optimise(&mut self, secret: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let model = self.model.clone();

        let body = OpenAIChatRequest {
            model,
            messages: vec![Message {
                role: "system".to_string(),
                content: draft_instructions(
                    &self.function,
                    self.add_comments,
                    &self.optional_context,
                ),
            }],
        };

        let url = Url::parse(COMPLETION_URL).unwrap();

        return match client.post(url).bearer_auth(secret).json(&body).build() {
            Ok(req) => match client.execute(req).await {
                Ok(resp) => match resp.json::<OpenAIChatResponse>().await {
                    Ok(parsed_resp) => {
                        let content = parsed_resp.choices[0].message.content.clone();
                        if content == "OPTIMAL" {
                            return Err("optimal".to_string());
                        }

                        return Ok(content);
                    }
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };
    }
    fn edit_source_file(&mut self, suggestion: &[u8]) {
        let parsed_suggestion = tree_sitter::Parser:
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Location of the source code file
    #[arg(short, long)]
    file_location: String,

    /// Name of the function that will be searched for in the file at the given file_location
    #[arg(short, long)]
    function_name: String,

    /// The `bat` theme. Check out https://github.com/sharkdp/bat/tree/master/assets/themes for a list of available themes
    #[arg(short, long)]
    theme: String,

    /// The OpenAI model. Check out https://platform.openai.com/docs/models/overview
    #[arg(short, long)]
    model: String,

    /// Anything else you would like to tell your chosen LLM
    #[arg(short, long)]
    optional_context: String,

    /// Setting this option to true will print out the suggestion without a confirmation prompt
    #[arg(short, long)]
    dry_run: bool,

    /// Setting this option to true will print and automatically overwrite the function in the source file
    #[arg(short, long)]
    skip_prompt: bool,

    /// Should the new code have comments?
    #[arg(short, long)]
    add_comments: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let secret = env::var(OPENAI_API_KEY_ENV_VAR).unwrap();

    let mut opt = Optimizer::new(
        &args.file_location,
        &args.function_name,
        &args.optional_context,
        &args.theme,
        &args.model,
        args.dry_run,
        args.add_comments,
        args.skip_prompt,
    );
    let output = opt.optimise(&secret).await.unwrap();

    bat::PrettyPrinter::new()
        .input_from_bytes(output.as_bytes())
        .language("rust")
        .vcs_modification_markers(true)
        .print()
        .unwrap();

    if !opt.skip_prompt {
        Confirm::new("Apply the suggestion?")
            .with_default(false)
            .prompt()
            .unwrap();
    }

    if !opt.dry_run {
        opt.edit_source_file(output.as_bytes())
    }
}
