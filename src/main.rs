use aws_sso::aws_sso::AwsSsoWorkflow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AwsSsoWorkflow::new(
        "https://webstar34.awsapps.com/start".to_string(),
        "eu-west-1".to_string(),
    )
    .run()
    .await
    .map_err(|e| e.into())
}
