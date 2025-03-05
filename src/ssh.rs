use std::process::Command;

pub fn execute_ssh_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("sh").arg("-c").arg(command).spawn()?.wait()?;
    Ok(())
}
