use skim::prelude::*;
use std::io::Cursor;

pub fn select_instance(instances: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let options = SkimOptionsBuilder::default()
        .height("10".to_string())
        .prompt("Select an EC2 instance: ".to_string())
        .multi(false)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(instances.join("\n")));

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    if let Some(selected) = selected_items.first() {
        Ok(selected.output().to_string())
    } else {
        Err("No instance selected".into())
    }
}
