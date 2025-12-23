use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub style_influences: Vec<String>,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Stage {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub processor_hint: Option<String>,
    #[serde(default)]
    pub inputs: Vec<Input>,
    #[serde(default)]
    pub outputs: Vec<Output>,
    pub operation: Operation,
    #[serde(default)]
    pub tasks: Vec<Task>, // For sequence or loop operations with nested tasks
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)] // This allows deserializing based on the content of the enum variants
pub enum Operation {
    FunctionCall(FunctionCallOperation),
    Loop(LoopOperation),
    Sequence(SequenceOperation),
    Switch(SwitchOperation),
    Shell(ShellCommandOperation), // New operation for running shell commands
    // Add other operation types as needed
    #[serde(untagged)]
    Unknown(toml::Value), // Catch-all for unknown operation types
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwitchOperation {
    #[serde(rename = "type")]
    pub op_type: String, // Should be "switch"
    pub match_on: String, // The context key to match against
    pub cases: HashMap<String, Vec<Task>>, // Map of value to match to a list of tasks
    #[serde(default)]
    pub default: Option<Vec<Task>>, // Optional default tasks if no case matches
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionCallOperation {
    #[serde(rename = "type")]
    pub op_type: String, // Should be "function_call"
    pub function: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub recursive: Option<bool>,
    #[serde(default)]
    pub output_base: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoopOperation {
    #[serde(rename = "type")]
    pub op_type: String, // Should be "loop"
    pub over: String,
    pub loop_var: String,
    pub tasks: Vec<Task>, // Nested tasks within the loop
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SequenceOperation {
    #[serde(rename = "type")]
    pub op_type: String, // Should be "sequence"
    pub tasks: Vec<Task>, // Nested tasks within the sequence
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ShellCommandOperation {
    #[serde(rename = "type")]
    pub op_type: String, // Should be "shell"
    pub command: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default = "default_true")]
    pub capture_output: bool,
    #[serde(default)]
    pub error_on_failure: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub name: String,
    pub operation: Operation,
    #[serde(default)]
    pub inputs: Vec<Input>,
    #[serde(default)]
    pub outputs: Vec<Output>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Input {
    pub name: String,
    #[serde(default)]
    pub from_stage: Option<String>,
    #[serde(default)]
    pub from_task: Option<String>,
    pub output_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Output {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: String,
    pub description: String,
}

impl Workflow {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let workflow: Workflow = toml::from_str(&content)?;
        Ok(workflow)
    }
}

// Top-level structure for goal.toml to include the original-goal field
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GoalConfig {
    #[serde(rename = "original-goal")]
    pub original_goal: Option<String>,
    pub workflow: Workflow,
}

impl GoalConfig {
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: GoalConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
