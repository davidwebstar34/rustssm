use cloud_sso::create_workflow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the cloud provider ("aws" or "google")
    let provider = "aws"; // Change this to "google" for GCP

    // Create the workflow
    let workflow = create_workflow(provider)?;

    // Run the workflow
    workflow.run()?;

    Ok(())
}
