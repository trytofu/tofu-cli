use colored::Colorize;
use comfy_table::{
    Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

pub fn success(s: impl std::fmt::Display) {
    let s = s.to_string();
    let output = s.bright_green().bold();
    println!("{}", output)
}

pub fn error(s: impl std::fmt::Display) {
    let s = s.to_string();
    eprintln!("{}", s.bright_red().bold())
}

pub fn warning(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.bright_yellow().bold())
}

pub fn command(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.bright_blue().bold())
}

pub fn info(s: impl std::fmt::Display) {
    let s = s.to_string();
    println!("{}", s.blue().bold())
}

pub fn kv_table(rows: Vec<(&str, String)>) -> Table {
    kv_table_cells(
        rows.into_iter()
            .map(|(key, value)| (key, Cell::new(value)))
            .collect(),
    )
}

pub fn kv_table_cells(rows: Vec<(&str, Cell)>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    for (key, value) in rows {
        table.add_row(vec![Cell::new(key).add_attribute(Attribute::Bold), value]);
    }

    table
}
