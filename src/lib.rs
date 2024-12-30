pub mod aws;
use std::error::Error;

pub fn create_workflow() -> Result<Box<dyn Workflow>, Box<dyn Error>> {
    Ok(Box::new(aws::AwsSsoWorkflow::default()))
}

pub trait Workflow {
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
