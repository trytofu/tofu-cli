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
