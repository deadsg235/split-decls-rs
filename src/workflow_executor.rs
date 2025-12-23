use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::goal_parser::{Workflow, Stage, Operation, FunctionCallOperation, LoopOperation, SequenceOperation, Task, Input, Output, SwitchOperation};
use crate::eager_splitter;
use crate::setup_crate_paths;
use crate::crate_finder::{self, CrateInfo};
use crate::patch_config::PatchConfig;
use crate::generate_new_cargotoml;
use crate::generate_new_workspace;
use split_decls_types::SplitDeclsConfig;

pub struct WorkflowExecutor {
    verbose: bool,
    dry_run: bool,
    global_config: SplitDeclsConfig,
    // Add other fields as needed for context (e.g., outputs from previous stages)
    context: HashMap<String, toml::Value>,
}

impl WorkflowExecutor {
    pub fn new(verbose: bool, dry_run: bool, global_config: SplitDeclsConfig) -> Self {
        WorkflowExecutor {
            verbose,
            dry_run,
            global_config,
            context: HashMap::new(),
        }
    }

    pub fn execute(&mut self, workflow: &Workflow) -> Result<()> {
        if self.verbose {
            println!("Executing workflow: {}", workflow.name);
            println!("Description: {}", workflow.description);
        }

        for stage in &workflow.stages {
            self.execute_stage(stage)?;
        }

        Ok(())
    }

    fn execute_stage(&mut self, stage: &Stage) -> Result<()> {
        if self.verbose {
            println!("\nExecuting stage: {}", stage.name);
            println!("Stage description: {}", stage.description);
        }

        let mut processed_crates_accumulator: Option<Vec<toml::Value>> = None;

        // If this is the "Extract and Copy Declarations" stage, prepare to collect processed crate info
        if stage.name == "Extract and Copy Declarations" {
            processed_crates_accumulator = Some(Vec::new());
        }

        match &stage.operation {
            Operation::FunctionCall(op) => self.execute_function_call(op, &stage.inputs, &stage.outputs)?,
            Operation::Loop(op) => {
                // If it's the "Extract and Copy Declarations" loop, we need to collect the processed crate info
                let collection_toml_value = self.context.get(&op.over)
                    .context(format!("Loop variable '{}' not found in context", op.over))?;
                
                let collection_to_loop: Vec<toml::Value> = collection_toml_value.as_array()
                    .context(format!("Loop variable '{}' is not an array", op.over))?
                    .clone();

                for item_toml_value in collection_to_loop {
                    // Set the loop variable in context
                    self.context.insert(op.loop_var.clone(), item_toml_value.clone());
                    if self.verbose {
                        println!("    Loop iteration: {} = {:?}", op.loop_var, item_toml_value);
                    }
                    for task in &op.tasks {
                        self.execute_task(task, &stage.inputs, &stage.outputs)?;
                    }

                    // After processing tasks for a crate, if we are in the "Extract and Copy Declarations" stage,
                    // collect the crate_info (which is item_toml_value)
                    if let Some(accumulator) = &mut processed_crates_accumulator {
                        accumulator.push(item_toml_value.clone());
                    }
                }
            },
            Operation::Sequence(op) => self.execute_sequence(op, &stage.inputs, &stage.outputs)?,
            Operation::Switch(op) => self.execute_switch(op, &stage.inputs, &stage.outputs)?,
            Operation::Unknown(value) => {
                anyhow::bail!("Unknown operation type encountered in stage {}: {:?}", stage.name, value);
            }
        }

        // If we collected processed crate info, store it in the context
        if let Some(accumulator) = processed_crates_accumulator {
            if stage.name == "Extract and Copy Declarations" {
                self.context.insert("all_processed_crates_info".to_string(), toml::Value::Array(accumulator));
                if self.verbose {
                    println!("  Aggregated {} processed crate infos as 'all_processed_crates_info'.", 
                             self.context["all_processed_crates_info"].as_array().map_or(0, |arr| arr.len()));
                }
            }
        }
        Ok(())
    }

    fn execute_function_call(&mut self, op: &FunctionCallOperation, inputs: &[Input], outputs: &[Output]) -> Result<()> {
        if self.verbose {
            println!("  Executing function call: {} with args {:?}", op.function, op.args);
        }

        let resolved_args: Vec<String> = op.args.iter()
            .map(|arg| self.resolve_arg_value(arg))
            .collect::<Result<Vec<String>>>()?;
        
        let resolved_output_base = if let Some(ob) = &op.output_base {
            Some(self.resolve_arg_value(ob)?)
        } else {
            None
        };

        match op.function.as_str() {
            "find_crates" => {
                if self.verbose {
                    println!("  (Calling crate_finder::find_crates)");
                }
                let base_path_str = resolved_args.get(0).context("find_crates expects a base_path argument")?;
                let base_path = PathBuf::from(base_path_str);
                let discovered_crates = crate_finder::find_crates(&base_path)?;

                // Store output in context
                if let Some(output_def) = outputs.iter().find(|o| o.name == "discovered_crates") {
                    let toml_value = toml::Value::Array(
                        discovered_crates.into_iter().map(|ci| {
                            let mut table = toml::Table::new();
                            table.insert("name".to_string(), toml::Value::String(ci.name));
                            table.insert("original_path".to_string(), toml::Value::String(ci.original_path.to_string_lossy().into_owned()));
                            toml::Value::Table(table)
                        }).collect()
                    );
                    self.context.insert(output_def.name.clone(), toml_value);
                    if self.verbose {
                        println!("  Discovered {} crates.", self.context[&output_def.name].as_array().map_or(0, |arr| arr.len()));
                    }
                }
            },
            "extract_declarations" => {
                if self.verbose {
                    println!("  (Calling eager_splitter::extract_declarations_to_map)");
                }
                let crate_path_str = resolved_args.get(0).context("extract_declarations expects a crate_path argument")?;
                let crate_path = PathBuf::from(crate_path_str);
                
                let paths = setup_crate_paths(&crate_path)?;
                let extracted_decls = eager_splitter::extract_declarations_to_map(&paths)?;

                if let Some(output_def) = outputs.iter().find(|o| o.name == "extracted_decls") {
                    let mut toml_decls_map = toml::Table::new(); // toml::Table is an alias for toml::Map
                    for (name, tokens_str) in extracted_decls {
                        toml_decls_map.insert(name, toml::Value::String(tokens_str.to_string()));
                    }
                    self.context.insert(output_def.name.clone(), toml::Value::Table(toml_decls_map));
                    if self.verbose {
                        println!("  Extracted {} declarations.", self.context[&output_def.name].as_table().map_or(0, |tbl| tbl.len()));
                    }
                }
            },
            "copy_to_output" => {
                if self.verbose {
                    println!("  (Calling eager_splitter::copy_declarations_to_output)");
                }
                let crate_name = resolved_args.get(0).context("copy_to_output expects crate_name as the first argument")?;
                let declarations_context_key = resolved_args.get(1).context("copy_to_output expects declarations context key as the second argument")?;
                let output_base_path_str = resolved_args.get(2).context("copy_to_output expects output_base_path as the third argument")?;
                let output_base_path = PathBuf::from(output_base_path_str);

                let declarations_toml = self.context.get(declarations_context_key)
                    .context(format!("Declarations '{}' not found in context for copy_to_output", declarations_context_key))?
                    .as_table()
                    .context(format!("Declarations '{}' in context is not a table for copy_to_output", declarations_context_key))?;
                
                let mut declarations_map: HashMap<String, String> = HashMap::new();
                for (name, value) in declarations_toml {
                    declarations_map.insert(name.clone(), value.as_str().context("Declaration value is not a string")?.to_string());
                }

                eager_splitter::copy_declarations_to_output(crate_name, &declarations_map, &output_base_path)?;
                if self.verbose {
                    println!("  Copied {} declarations for crate '{}' to {}", declarations_map.len(), crate_name, output_base_path.display());
                }
            },
            "generate_workspace_root_toml" => {
                if self.verbose {
                    println!("  (Calling generate_new_workspace::generate_root_toml)");
                }
                let output_dir_str = resolved_args.get(0).context("generate_workspace_root_toml expects output_dir as the first argument")?;
                let processed_crates_context_key = resolved_args.get(1).context("generate_workspace_root_toml expects processed_crates context key as the second argument")?;
                
                let output_dir = PathBuf::from(output_dir_str);

                let processed_crates_toml = self.context.get(processed_crates_context_key)
                    .context(format!("Processed crates '{}' not found in context for generate_workspace_root_toml", processed_crates_context_key))?
                    .as_array()
                    .context(format!("Processed crates '{}' in context is not an array for generate_workspace_root_toml", processed_crates_context_key))?;
                
                let processed_crate_paths: Vec<PathBuf> = processed_crates_toml.iter()
                    .filter_map(|v| v.as_str())
                    .map(PathBuf::from)
                    .collect();

                generate_new_workspace::generate_root_toml(&output_dir, &processed_crate_paths, self.dry_run)?;
                if self.verbose {
                    println!("  Generated workspace root Cargo.toml in {}", output_dir.display());
                }
            },
            "generate_crate_toml" => {
                if self.verbose {
                    println!("  (Calling generate_new_cargotoml::generate_new_cargotoml)");
                }
                let output_crate_dir_str = resolved_args.get(0).context("generate_crate_toml expects output_crate_dir as the first argument")?;
                let crate_info_context_key = resolved_args.get(1).context("generate_crate_toml expects crate_info context key as the second argument")?;
                
                let output_crate_dir = PathBuf::from(output_crate_dir_str);

                let crate_info_toml = self.context.get(crate_info_context_key)
                    .context(format!("Crate info '{}' not found in context for generate_crate_toml", crate_info_context_key))?
                    .as_table()
                    .context(format!("Crate info '{}' in context is not a table for generate_crate_toml", crate_info_context_key))?;
                
                let crate_name = crate_info_toml.get("name")
                    .context("Crate info missing 'name' field")?
                    .as_str()
                    .context("Crate name is not a string")?
                    .to_string();

                let original_crate_path_str = crate_info_toml.get("original_path")
                    .context("Crate info missing 'original_path' field")?
                    .as_str()
                    .context("Original crate path is not a string")?
                    .to_string();
                let original_crate_root_path = PathBuf::from(original_crate_path_str);
                let original_cargo_toml_path = original_crate_root_path.join("Cargo.toml");
                let output_cargo_toml_path = output_crate_dir.join("Cargo.toml");

                // Dummy PatchConfig for now - this should ideally be passed through or loaded
                // from a common config.
                let dummy_patch_config = PatchConfig::default();

                crate::generate_new_cargotoml::generate_new_cargotoml(
                    &original_cargo_toml_path,
                    &output_cargo_toml_path,
                    &original_crate_root_path,
                    &self.global_config,
                    &dummy_patch_config,
                    self.dry_run,
                )?;
                if self.verbose {
                    println!("  Generated Cargo.toml for crate '{}' in {}", crate_name, output_crate_dir.display());
                }
            },
            _ => {
                anyhow::bail!("Unknown function call: {}", op.function);
            }
        }

        // --- End Placeholder ---
        
        Ok(())
    }

    fn execute_loop(&mut self, op: &LoopOperation, inputs: &[Input], outputs: &[Output]) -> Result<()> {
        if self.verbose {
            println!("  Executing loop: over {} as {}", op.over, op.loop_var);
        }

        // Retrieve the collection to loop over from context
        let collection_toml_value = self.context.get(&op.over)
            .context(format!("Loop variable '{}' not found in context", op.over))?;
        
        let collection_to_loop: Vec<toml::Value> = collection_toml_value.as_array()
            .context(format!("Loop variable '{}' is not an array", op.over))?
            .clone();

        for item_toml_value in collection_to_loop {
            // Set the loop variable in context
            self.context.insert(op.loop_var.clone(), item_toml_value.clone());
            if self.verbose {
                println!("    Loop iteration: {} = {:?}", op.loop_var, item_toml_value);
            }
            for task in &op.tasks {
                self.execute_task(task, inputs, outputs)?;
            }
        }
        Ok(())
    }

    fn execute_sequence(&mut self, op: &SequenceOperation, inputs: &[Input], outputs: &[Output]) -> Result<()> {
        if self.verbose {
            println!("  Executing sequence of {} tasks", op.tasks.len());
        }
        for task in &op.tasks {
            self.execute_task(task, inputs, outputs)?;
        }
        Ok(())
    }

    fn execute_switch(&mut self, op: &SwitchOperation, inputs: &[Input], outputs: &[Output]) -> Result<()> {
        if self.verbose {
            println!("  Executing switch: matching on '{}'", op.match_on);
        }

        let value_to_match = self.context.get(&op.match_on)
            .context(format!("Match variable '{}' not found in context for switch operation", op.match_on))?;

        let mut matched = false;
        for (case_value_str, tasks) in &op.cases {
            // Convert context value to string for comparison
            let context_value_str = value_to_match.as_str()
                .context(format!("Context value for '{}' is not a string, cannot match against cases", op.match_on))?;

            if context_value_str == case_value_str {
                if self.verbose {
                    println!("    Case matched: '{}'", case_value_str);
                }
                for task in tasks {
                    self.execute_task(task, inputs, outputs)?;
                }
                matched = true;
                break;
            }
        }

        if !matched {
            if let Some(default_tasks) = &op.default {
                if self.verbose {
                    println!("    No case matched, executing default tasks.");
                }
                for task in default_tasks {
                    self.execute_task(task, inputs, outputs)?;
                }
            } else {
                if self.verbose {
                    println!("    No case matched and no default tasks to execute.");
                }
            }
        }

        Ok(())
    }

    fn execute_task(&mut self, task: &Task, parent_inputs: &[Input], parent_outputs: &[Output]) -> Result<()> {
        if self.verbose {
            println!("    Executing task: {}", task.name);
        }

        // Inputs and outputs for tasks can be handled similarly to stages,
        // potentially merging with parent stage's inputs/outputs or overriding
        let task_inputs = if task.inputs.is_empty() { parent_inputs } else { &task.inputs };
        let task_outputs = if task.outputs.is_empty() { parent_outputs } else { &task.outputs };

        match &task.operation {
            Operation::FunctionCall(op) => self.execute_function_call(op, task_inputs, task_outputs)?,
            Operation::Loop(op) => self.execute_loop(op, task_inputs, task_outputs)?,
            Operation::Sequence(op) => self.execute_sequence(op, task_inputs, task_outputs)?,
            Operation::Switch(op) => self.execute_switch(op, task_inputs, task_outputs)?,
            Operation::Unknown(value) => {
                anyhow::bail!("Unknown operation type encountered in task {}: {:?}", task.name, value);
            }
        }
        Ok(())
    }

    // Helper to resolve argument values from context, supporting basic templating
    fn resolve_arg_value(&self, arg: &str) -> Result<String> {
        if arg.starts_with("{{") && arg.ends_with("}}") {
            let path = arg[2..(arg.len() - 2)].trim(); // e.g., "crate_path" or "crate_info.name"
            
            if let Some(value) = self.context.get(path) {
                // If it's a simple key, return its string representation
                if value.is_str() {
                    Ok(value.as_str().unwrap().to_string())
                } else if value.is_array() {
                    // If it's an array, convert to a comma-separated string for now.
                    // This might need more sophisticated handling later.
                    let arr_str: Vec<String> = value.as_array().unwrap().iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    Ok(format!("[{}]", arr_str.join(", ")))
                } else {
                    // Try to handle "crate_info.name" style access
                    let parts: Vec<&str> = path.split('.').collect();
                    if parts.len() == 2 {
                        let obj_name = parts[0];
                        let field_name = parts[1];

                        if let Some(obj_value) = self.context.get(obj_name) {
                            if let Some(obj_table) = obj_value.as_table() {
                                if let Some(field_value) = obj_table.get(field_name) {
                                    if field_value.is_str() {
                                        return Ok(field_value.as_str().unwrap().to_string());
                                    }
                                }
                            }
                        }
                    }
                    anyhow::bail!("Unsupported context value type or complex path for arg: {}", arg);
                }
            } else {
                anyhow::bail!("Context variable '{}' not found for arg: {}", path, arg);
            }
        } else {
            // Not a template string, return as is
            Ok(arg.to_string())
        }
    }
}
