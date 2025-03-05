//! # rustssm
//!
//! A Rust-based AWS SSM session helper.
//!
//! This tool allows users to interactively select AWS EC2 instances that support
//! AWS Systems Manager (SSM) and start secure sessions without needing SSH access.
//!
//! ## Features
//! - Interactive instance selection via fuzzy finder (`skim`).
//! - Secure SSM session handling (start/terminate).
//! - AWS credential and region configuration.

mod aws_config;
mod ec2;
mod interactive;
mod ssm;

use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use std::error::Error;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let region = "eu-west-1";
        let aws_config = aws_config::configure_aws(Some(region.to_string())).await;

        let ec2_client = Ec2Client::new(&aws_config);
        let ssm_client = SsmClient::new(&aws_config);

        println!("Fetching available EC2 instances...");
        let instances = ec2::list_ec2_instances(&ec2_client).await?;

        if instances.is_empty() {
            println!("No EC2 instances found with running state.");
            return Ok(());
        }

        let selected_instance = interactive::select_instance(&instances)?;
        println!("Selected instance: {}", selected_instance);

        let session_id = ssm::start_ssm_session(&ssm_client, &selected_instance).await?;
        println!("SSM session started: {}", session_id);

        ssm::execute_ssm_session(&selected_instance, region)?;
        ssm::terminate_ssm_session(&ssm_client, &session_id).await?;
        println!("SSM session terminated: {}", session_id);

        Ok(())
    })
}
