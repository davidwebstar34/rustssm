use std::error::Error;

pub struct GcpWorkflow {
    project_id: String,
}

impl GcpWorkflow {
    pub fn new(project_id: String) -> Self {
        Self { project_id }
    }

    pub fn run_workflow(&self) -> Result<(), Box<dyn Error>> {
        println!("To still be implemented: {}", self.project_id);
        // Add GCP-specific logic here
        Ok(())
    }
}

// Implement the common Workflow trait
impl super::Workflow for GcpWorkflow {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        self.run_workflow()
    }
}
