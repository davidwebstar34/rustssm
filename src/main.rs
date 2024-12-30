use aws_sso::create_workflow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = create_workflow()?; // Make the workflow mutable
    workflow.run()?; // Call the run method
    Ok(())
}
