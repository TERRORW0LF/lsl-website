use core::str;

pub fn escape_regex(input: &str) -> String {
    let mut buf = String::new();
    buf.reserve(input.len());
    for ch in input.chars() {
        match ch {
            '\\' | '^' | '$' | '*' | '+' | '?' | '{' | '}' | '(' | ')' | '[' | ']' | '|' => buf.push('\\'),
            _ => (),
        };
        buf.push(ch);
    }
    buf
}
