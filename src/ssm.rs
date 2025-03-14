use aws_sdk_ssm::Client as SsmClient;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::process::{Command, Stdio};

pub async fn execute_ssm_session_with_plugin(
    client: &SsmClient,
    instance_id: &str,
    region: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.start_session().target(instance_id).send().await?;

    let session_id = response
        .session_id()
        .ok_or("Failed to get session ID from response")?;

    let stream_url = response
        .stream_url()
        .ok_or("Failed to get stream URL from response")?;

    let token_value = response
        .token_value()
        .ok_or("Failed to get token value from response")?;

    let status = Command::new("session-manager-plugin")
        .arg(session_metadata_json(
            &session_id,
            &stream_url,
            &token_value,
        ))
        .arg(region)
        .arg("StartSession")
        .arg("") // empty AWS profile means default
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        println!("Interactive session ended successfully.");
        Ok(())
    } else {
        Err("session-manager-plugin failed to execute session".into())
    }
}

fn session_metadata_json(session_id: &str, stream_url: &str, token_value: &str) -> String {
    serde_json::json!({
        "SessionId": session_id,
        "StreamUrl": stream_url,
        "TokenValue": token_value,
    })
    .to_string()
}

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

pub async fn send_ssh_key_via_ssm(
    client: &SsmClient,
    instance_id: &str,
    ssh_public_key_path: &str,
    username: &str,
) -> Result<(), Box<dyn Error>> {
    let ssh_public_key = fs::read_to_string(ssh_public_key_path)?.trim().to_string();

    let command = format!(
        "sudo -u {1} bash -c 'mkdir -p /home/{1}/.ssh && chmod 700 /home/{1}/.ssh && \
        touch /home/{1}/.ssh/authorized_keys && chmod 600 /home/{1}/.ssh/authorized_keys && \
        grep -qxF \"{0}\" /home/{1}/.ssh/authorized_keys || echo \"{0}\" >> /home/{1}/.ssh/authorized_keys'",
        ssh_public_key, username
    );

    client
        .send_command()
        .document_name("AWS-RunShellScript")
        .instance_ids(instance_id)
        .parameters("commands", vec![command])
        .comment("Append SSH public key to authorized_keys")
        .send()
        .await?;

    Ok(())
}

pub async fn start_port_forwarding_ssm_session(
    client: &SsmClient,
    instance_id: &str,
    region: &str,
    local_port: u16,
    remote_port: u16,
) -> Result<(), Box<dyn Error>> {
    // Start an SSM session for port forwarding
    let response = client
        .start_session()
        .target(instance_id)
        .document_name("AWS-StartPortForwardingSession")
        .parameters("portNumber", vec![remote_port.to_string()])
        .parameters("localPortNumber", vec![local_port.to_string()])
        .send()
        .await?;

    // Extract session details
    let session_id = response.session_id().ok_or("Missing session ID")?;
    let stream_url = response.stream_url().ok_or("Missing stream URL")?;
    let token_value = response.token_value().ok_or("Failed to get token")?;

    // Correct JSON structure for session-manager-plugin (must include Parameters!)
    let session_metadata = json!({
        "SessionId": session_id,
        "StreamUrl": stream_url,
        "TokenValue": token_value,
    });

    let parameters_json = json!({
        "portNumber": [remote_port.to_string()],
        "localPortNumber": [local_port.to_string()]
    });

    // Pass correct arguments to session-manager-plugin (6 arguments total)
    let status = Command::new("session-manager-plugin")
        .arg(session_metadata.to_string()) // Session metadata JSON
        .arg(region)
        .arg("StartSession") // <-- Must remain "StartSession"
        .arg("") // empty for default AWS profile
        .arg(
            json!({                             // Additional argument: parameters JSON
                "Target": instance_id,
                "DocumentName": "AWS-StartPortForwardingSession",
                "Parameters": parameters_json
            })
            .to_string(),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        println!(
            "Port forwarding established: connect via ssh -p {} username@localhost",
            local_port
        );
        Ok(())
    } else {
        Err("session-manager-plugin failed to execute port forwarding session".into())
    }
}
