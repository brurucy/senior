use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;
use tree_sitter::Parser;
use tree_sitter_traversal::{traverse_tree, Order};

const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
const COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

struct Optimizer {
    file_name: String,
    function_name: String,
    source_file: String,
    context: String,
    model: String,
}

impl Default for Optimizer {
    fn default() -> Self {
        return Self {
            file_name: Default::default(),
            function_name: Default::default(),
            source_file: Default::default(),
            context: Default::default(),
            model: "gpt-3.5-turbo".to_string(),
        };
    }
}

impl Optimizer {
    fn new(file_name: &str, function_name: &str, optional_context: &str) -> Self {
        let mut chat_gpt: Optimizer = Default::default();

        chat_gpt.file_name = file_name.to_string();
        chat_gpt.function_name = function_name.to_string();
        chat_gpt.source_file = read_to_string(&chat_gpt.file_name).unwrap();
        chat_gpt.context = optional_context.to_string();

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(&chat_gpt.context, None).unwrap();
        let source = chat_gpt.context.clone().into_bytes();
        let function_node = traverse_tree(&tree, Order::Pre)
            .find(|node| {
                let node_kind = node.kind();
                if node_kind == "function_item" {
                    let node_function_name = node.child_by_field_name("name").unwrap();
                    return node_function_name.utf8_text(&source).unwrap() == function_name;
                }

                return false;
            })
            .unwrap()
            .utf8_text(&source)
            .unwrap();
        chat_gpt.context = function_node.to_string();

        return chat_gpt;
    }
    fn draft_instructions(&self) -> String {
        let instructions = format!(
            r#"You are a program optimizer.

This is the starting point:
{}

Your task is to optimise the function named: {}

Here is some context: {}

Optimise it without changing the type signature, and only propose small, incremental changes.

Respond only with the optimised code, without any comments whatsover, at all.

If the code cannot be optimised further, respond with "OPTIMAL".
"#,
            self.source_file, self.function_name, self.context
        );

        return instructions;
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
    index: usize,
    message: Message,
}

#[derive(Deserialize, Debug)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatResponseChoice>,
}

impl Optimizer {
    async fn optimise(&mut self, secret: &str) -> Option<String> {
        let client = reqwest::Client::new();
        let model = self.model.clone();

        let body = OpenAIChatRequest {
            model,
            messages: vec![Message {
                role: "system".to_string(),
                content: self.draft_instructions(),
            }],
        };

        let url = Url::parse(COMPLETION_URL).unwrap();

        let req = client
            .post(url)
            .bearer_auth(secret)
            .json(&body)
            .build()
            .unwrap();

        let resp = client
            .execute(req)
            .await
            .unwrap()
            .json::<OpenAIChatResponse>()
            .await
            .unwrap();

        let content = resp.choices[0].message.content.clone();

        if content == "OPTIMAL" {
            return None;
        }

        return Some(content);
    }
}

#[tokio::main]
async fn main() {
    let secret = env::var(OPENAI_API_KEY_ENV_VAR).unwrap();

    let mut opt = Optimizer::new("../indexset/src/lib.rs", "insert", "");
    let output = opt.optimise(&secret).await.unwrap();

    bat::PrettyPrinter::new()
        .input_from_bytes(output.as_bytes())
        .language("rust")
        .print()
        .unwrap();
}
