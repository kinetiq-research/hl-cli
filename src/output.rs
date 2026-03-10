use comfy_table::{presets::UTF8_FULL_CONDENSED, ContentArrangement, Table};
use serde::Serialize;

use crate::error::CliError;

pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    table
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}

pub fn print_table(table: Table) {
    println!("{table}");
}
