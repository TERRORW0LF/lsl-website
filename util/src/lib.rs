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

pub trait OptResToBool {
    fn opt_res_to_bool(&self) -> bool;
}

impl<E> OptResToBool for Option<Result<bool, E>> {
    fn opt_res_to_bool(&self) -> bool {
        self.as_ref().map(|r| r.as_ref().ok()).flatten().is_some_and(|b| *b)
    }
}

#[cfg(feature = "ssr")]
pub mod ssr {
    use http::HeaderValue;

    pub fn redirect(redirect: Option<String>) {
        if let Some(red) = redirect {
            let red = if red.starts_with("/") { red } else { format!("/{}", red) };
            if let Some(re) = HeaderValue::from_str(&red).ok() {
                leptos_axum::redirect(re.to_str().unwrap_or("/"), false);
            };
        }
    }
}
