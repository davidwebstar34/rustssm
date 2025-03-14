use aws_sdk_ec2::types::Filter;
use aws_sdk_ec2::Client as Ec2Client;

pub async fn list_ec2_instances(
    client: &Ec2Client,
    tag_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let resp = client
        .describe_instances()
        .filters(Filter::builder().name(tag_name).values("running").build())
        .send()
        .await?;

    let instances: Vec<String> = resp
        .reservations()
        .iter()
        .flat_map(|res| res.instances())
        .filter_map(|inst| inst.instance_id().map(|id| id.to_string()))
        .collect();

    Ok(instances)
}
