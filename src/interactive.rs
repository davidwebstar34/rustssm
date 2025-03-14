use inquire::{error::InquireError, Select};

pub fn select_instance(instances: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let selected = Select::new("Select an EC2 instance:", instances.to_vec()).prompt();

    match selected {
        Ok(instance_id) => Ok(instance_id),
        Err(InquireError::OperationCanceled) => Err("Selection cancelled by user".into()),
        Err(err) => Err(Box::new(err)),
    }
}
