use aws_sdk_ssm::Client as SsmClient;

pub async fn start_ssm_session(
    client: &SsmClient,
    instance_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client.start_session().target(instance_id).send().await?;

    if let Some(session_id) = response.session_id() {
        Ok(session_id.to_string())
    } else {
        Err("Failed to start session".into())
    }
}
pub async fn terminate_ssm_session(
    client: &SsmClient,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .terminate_session()
        .session_id(session_id)
        .send()
        .await?;
    Ok(())
}
