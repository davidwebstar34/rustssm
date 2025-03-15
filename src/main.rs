mod aws_config;
mod ec2;
mod interactive;
mod ssm;

use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "eu-west-1")]
    region: String,

    #[arg(long, default_value = "instance-state-name")]
    tag_name: String,

    #[arg(long, default_value = "ssm-user")]
    username: String,

    #[arg(long, default_value = "~/.ssh/id_rsa.pub")]
    ssh_key_path: String,

    #[arg(long, default_value = "2222")]
    local_port: u16,

    #[arg(long, default_value = "22")]
    remote_port: u16,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Connect,
    CopyKey,
    Tunnel,
    Notebook,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let aws_config = aws_config::configure_aws(Some(cli.region.clone())).await;
    let ec2_client = Ec2Client::new(&aws_config);
    let ssm_client = SsmClient::new(&aws_config);

    let instances = ec2::list_ec2_instances(&ec2_client, &cli.tag_name).await?;

    if instances.is_empty() {
        println!("No EC2 instances found with running state.");
        return Ok(());
    }

    let selected_instance = interactive::select_instance(&instances)?;

    match cli.action {
        Action::Connect => {
            ssm::connect_interactive_session(&ssm_client, &selected_instance, &cli.region).await?
        }
        Action::CopyKey => {
            ssm::copy_ssh_key(
                &ssm_client,
                &selected_instance,
                &cli.ssh_key_path,
                &cli.username,
            )
            .await?
        }
        Action::Tunnel => {
            ssm::establish_tunnel(
                &ssm_client,
                &selected_instance,
                &cli.region,
                cli.local_port,
                cli.remote_port,
            )
            .await?
        }
        Action::Notebook => {
            ssm::start_jupyter_notebook(
                &ssm_client,
                &selected_instance,
                &cli.region,
                &cli.username,
                cli.local_port,
                8888,
            )
            .await?
        }
    }

    Ok(())
}
