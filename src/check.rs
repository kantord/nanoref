use crate::checksum::{check_chars, SALT_V1};
use crate::search;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

const BROAD_RE: &str = r"\bnref-[A-Za-z0-9_-]{10,40}";
// Fewer context lines than ls — error output should be terse
const CONTEXT: usize = 2;

const PREFIX_LEN: usize = 5; // "nref-"
const CHECK_LEN: usize = 2;
const BODY_START: usize = PREFIX_LEN + CHECK_LEN;
const MARKER_LEN: usize = BODY_START + 21;

fn is_valid_shape(s: &str) -> bool {
    s.len() == MARKER_LEN
        && s.starts_with("nref-")
        && s[PREFIX_LEN..].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn highlight_match(line: &str, offset: usize, len: usize) -> String {
    let end = offset + len;
    format!(
        "{}{}{}",
        &line[..offset],
        (&line[offset..end]).red().bold(),
        &line[end..]
    )
}

pub fn run(path: &Path) -> Result<usize> {
    let hits = search::search(path, BROAD_RE)?;
    let hits_ctx = search::add_context(hits, CONTEXT, CONTEXT);
    let mut errors = 0;
    let mut last_path: Option<String> = None;
    let mut need_sep = false;

    for hit in hits_ctx {
        let error_label = if !is_valid_shape(&hit.text) {
            Some(format!("malformed: {}", hit.text))
        } else {
            let body = &hit.text[BODY_START..];
            let b = hit.text.as_bytes();
            let stored = (b[PREFIX_LEN] as char, b[PREFIX_LEN + 1] as char);
            let expected = check_chars(SALT_V1, body);
            if stored != expected {
                Some(format!("bad checksum: {}", hit.text))
            } else {
                None
            }
        };

        let label = match error_label {
            Some(l) => l,
            None => continue,
        };

        errors += 1;

        let path_changed = last_path.as_deref() != Some(&hit.path);
        if need_sep {
            println!("{}", "--".cyan());
        }
        if path_changed {
            println!("{}", hit.path.magenta().bold().underline());
            last_path = Some(hit.path.clone());
        }

        for (ln, text) in &hit.before {
            search::print_context_line("", *ln, text);
        }
        println!(
            "{} {}",
            "↪".cyan(),
            highlight_match(&hit.line_text, hit.line_offset, hit.text.len())
        );
        for (ln, text) in &hit.after {
            search::print_context_line("", *ln, text);
        }
        println!("{}", format!("error: {}", label).red().bold());
        need_sep = true;
    }

    Ok(errors)
}
