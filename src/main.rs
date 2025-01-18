use mongorph::{Crawler, ParseError};

fn fmt_err(s: &str, e: ParseError) -> String {
    let start = e.cursor;
    let lines: Vec<&str> = s.lines().collect();
    let mut line_num = 0;
    let mut col_num = 0;
    let mut current_index = 0;

    for (i, line) in lines.iter().enumerate() {
        if current_index + line.len() >= start {
            line_num = i + 1;
            col_num = start - current_index + 1;
            break;
        }
        current_index += line.len() + 1;
    }
    let error_line = lines.get(line_num - 1).unwrap_or(&"");
    let mut marker_line = String::new();
    marker_line.extend(" ".repeat(col_num - 1).chars());
    marker_line.push('^');

    format!(
        "ParseError::{:?}\n   --> line {}, column {}\n   |\n{:3}| {}\n   | {}\n",
        e.ty,
        line_num,
        col_num,
        line_num,
        error_line,
        marker_line
    )
}

fn main() {
    let s = String::from("match(branch == CSE)match(branch == CSE)");
    let mut crawler = Crawler::new(s.clone());
    match crawler.build() {
        Ok(_) => println!("AST: {:?}", crawler.ast),
        Err(e) => println!("{}", fmt_err(&s, e))
    };
}