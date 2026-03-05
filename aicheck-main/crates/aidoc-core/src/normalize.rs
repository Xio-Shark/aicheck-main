use regex::Regex;

pub fn normalize_text(input: &str, max_lines: usize) -> String {
    let ansi = Regex::new(r"\x1b\[[0-9;]*[A-Za-z]").expect("invalid ansi regex");
    let no_ansi = ansi.replace_all(input, "");
    let unified = no_ansi.replace("\r\n", "\n").replace('\r', "\n");

    let mut lines: Vec<&str> = unified.lines().collect();
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
    lines.join("\n")
}
