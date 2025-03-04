use std::process::Command;

pub fn generate_ssh_command(user: &str, instance_ip: &str) -> String {
    format!("ssh {}@{}", user, instance_ip)
}

pub fn execute_ssh_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("sh").arg("-c").arg(command).spawn()?.wait()?;
    Ok(())
}
