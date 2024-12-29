pub mod aws;
pub mod gcp;

use std::error::Error;

pub enum Provider {
    Aws,
    Google,
}

pub fn create_workflow(provider: &str) -> Result<Box<dyn Workflow>, Box<dyn Error>> {
    match provider.to_lowercase().as_str() {
        "aws" => Ok(Box::new(aws::AwsSsoWorkflow::new())),
        "google" => Ok(Box::new(gcp::GcpWorkflow::new(
            "gcp-project-id".to_string(),
        ))),
        _ => Err(format!("To still be implemented: {}", provider).into()),
    }
}

pub trait Workflow {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>>;
}
