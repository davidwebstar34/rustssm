use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_types::region::Region;

pub async fn configure_aws(region: Option<String>) -> aws_types::SdkConfig {
    let region_provider =
        RegionProviderChain::first_try(region.map(Region::new)).or_default_provider();

    aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .load()
        .await
}
