use colored::Colorize;

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
