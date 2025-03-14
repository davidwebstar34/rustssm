mod aws_config;
mod ec2;
mod interactive;
mod ssm;

use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let region = "eu-west-1";
    let tag_name: &str = "instance-state-name";
    let username: &str = "ssm-user";
    let ssh_key_path: &str = "/Users/webstar/.ssh/id_rsa.pub";
    let local_port: u16 = 2222;
    let remote_port: u16 = 22;

    let aws_config = aws_config::configure_aws(Some(region.to_string())).await;

    let ec2_client = Ec2Client::new(&aws_config);
    let ssm_client = SsmClient::new(&aws_config);

    let instances = ec2::list_ec2_instances(&ec2_client, &tag_name).await?;

    if instances.is_empty() {
        println!("No EC2 instances found with running state.");
        return Ok(());
    }

    let selected_instance = interactive::select_instance(&instances)?;

    ssm::send_ssh_key_via_ssm(&ssm_client, &selected_instance, &ssh_key_path, &username).await?;

    ssm::start_port_forwarding_ssm_session(
        &ssm_client,
        &selected_instance,
        &region,
        local_port,
        remote_port,
    )
    .await?;

    let session_id = ssm::start_ssm_session(&ssm_client, &selected_instance).await?;

    ssm::execute_ssm_session_with_plugin(&ssm_client, &selected_instance, region).await?;

    ssm::terminate_ssm_session(&ssm_client, &session_id).await?;

    Ok(())
}

// TODO:
// Add starting a jupyter notebook server on the EC2 instance using SSM or any kind of document
// Add clap to cater for command line arguments
