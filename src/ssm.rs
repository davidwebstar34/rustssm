use aws_sdk_ssm::Client as SsmClient;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::process::{Command, Stdio};

pub struct SessionInfo {
    pub session_id: String,
    pub stream_url: String,
    pub token_value: String,
}

pub async fn start_ssm_session_with_document(
    client: &SsmClient,
    instance_id: &str,
    document_name: &str,
    parameters: Option<serde_json::Value>,
) -> Result<SessionInfo, Box<dyn Error>> {
    let mut builder = client
        .start_session()
        .target(instance_id)
        .document_name(document_name);

    if let Some(serde_json::Value::Object(map)) = parameters {
        for (key, val) in map.iter() {
            let val_strs = val
                .as_array()
                .ok_or("Invalid parameter format")?
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect::<Vec<String>>();
            builder = builder.parameters(key, val_strs);
        }
    }

    let response = builder.send().await?;

    Ok(SessionInfo {
        session_id: response.session_id().unwrap().to_string(),
        stream_url: response.stream_url().unwrap().to_string(),
        token_value: response.token_value().unwrap().to_string(),
    })
}

pub fn run_session_manager_plugin(
    session_info: &SessionInfo,
    region: &str,
    document_name: &str,
    target: &str,
    parameters: Option<serde_json::Value>,
) -> Result<(), Box<dyn Error>> {
    let session_metadata = json!({
        "SessionId": session_info.session_id,
        "StreamUrl": session_info.stream_url,
        "TokenValue": session_info.token_value,
    });

    let additional_args = json!({
        "Target": target,
        "DocumentName": document_name,
        "Parameters": parameters.unwrap_or(json!({}))
    });

    let status = Command::new("session-manager-plugin")
        .arg(session_metadata.to_string())
        .arg(region)
        .arg("StartSession")
        .arg("") // default profile
        .arg(additional_args.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("session-manager-plugin execution failed".into())
    }
}

pub async fn run_ssm_command(
    client: &SsmClient,
    instance_id: &str,
    commands: Vec<String>,
    comment: &str,
) -> Result<(), Box<dyn Error>> {
    client
        .send_command()
        .document_name("AWS-RunShellScript")
        .instance_ids(instance_id)
        .parameters("commands", commands)
        .comment(comment)
        .send()
        .await?;
    Ok(())
}

pub async fn copy_ssh_key(
    client: &SsmClient,
    instance_id: &str,
    username: &str,
    ssh_key_path: &str,
) -> Result<(), Box<dyn Error>> {
    let expanded_path = shellexpand::tilde(ssh_key_path);
    let ssh_public_key = fs::read_to_string(expanded_path.as_ref())?.trim().to_string();

    let command = format!(
        "sudo -u {0} bash -c 'mkdir -p /home/{0}/.ssh && chmod 700 /home/{0}/.ssh && \
         touch /home/{0}/.ssh/authorized_keys && chmod 600 /home/{0}/.ssh/authorized_keys && \
         grep -qxF \"{1}\" /home/{0}/.ssh/authorized_keys || echo \"{1}\" >> /home/{0}/.ssh/authorized_keys'",
        username, ssh_public_key
    );

    run_ssm_command(client, instance_id, vec![command], "Append SSH public key").await
}

pub async fn connect_interactive_session(
    client: &SsmClient,
    instance_id: &str,
    region: &str,
) -> Result<(), Box<dyn Error>> {
    let params = Some(serde_json::json!({
        "command": ["bash"]
    }));
    let session_info = start_ssm_session_with_document(
        client,
        instance_id,
        "AWS-StartInteractiveCommand",
        params.clone(),
    )
    .await?;
    run_session_manager_plugin(
        &session_info,
        region,
        "AWS-StartInteractiveCommand",
        instance_id,
        params.clone(),
    )
}

pub async fn establish_tunnel(
    client: &SsmClient,
    instance_id: &str,
    region: &str,
    local_port: u16,
    remote_port: u16,
) -> Result<(), Box<dyn Error>> {
    let parameters = json!({
        "portNumber": [remote_port.to_string()],
        "localPortNumber": [local_port.to_string()]
    });

    let session_info = start_ssm_session_with_document(
        client,
        instance_id,
        "AWS-StartPortForwardingSession",
        Some(parameters.clone()),
    )
    .await?;

    run_session_manager_plugin(
        &session_info,
        region,
        "AWS-StartPortForwardingSession",
        instance_id,
        Some(parameters),
    )
}

pub async fn start_jupyter_notebook(
    client: &SsmClient,
    instance_id: &str,
    region: &str,
    username: &str,
    local_port: u16,
    remote_port: u16,
) -> Result<(), Box<dyn Error>> {
    run_ssm_command(
        client,
        instance_id,
        vec![format!("sudo -u {} nohup jupyter notebook --no-browser --ip=0.0.0.0 --port={} > ~/jupyter.log 2>&1 &", username, remote_port)],
        "Start Jupyter Notebook"
    ).await?;

    establish_tunnel(client, instance_id, region, local_port, remote_port).await
}
