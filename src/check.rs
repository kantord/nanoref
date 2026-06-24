use crate::checksum::{check_chars, SALT_V1};
use crate::search::{self, ContextWindow, HitWithContext, Span};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

const BROAD_RE: &str = r"\bnref-[A-Za-z0-9_-]{10,40}";
const CONTEXT: usize = 2;

const PREFIX_LEN: usize = 5;
const CHECK_LEN: usize = 2;
const BODY_START: usize = PREFIX_LEN + CHECK_LEN;
const MARKER_LEN: usize = BODY_START + 21;

fn is_valid_shape(s: &str) -> bool {
    s.len() == MARKER_LEN
        && s.starts_with("nref-")
        && s[PREFIX_LEN..]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Returns `(error_label, candidate_len)` or `None` if the marker is valid.
/// Truncates greedy `BROAD_RE` matches to `MARKER_LEN` before validating.
fn validate_hit(text: &str) -> Option<(String, usize)> {
    let len = text.len().min(MARKER_LEN);
    let candidate = &text[..len];
    if is_valid_shape(candidate) {
        let body = &candidate[BODY_START..];
        let b = candidate.as_bytes();
        let stored = (b[PREFIX_LEN] as char, b[PREFIX_LEN + 1] as char);
        let expected = check_chars(SALT_V1, body);
        (stored != expected).then(|| (format!("bad checksum: {candidate}"), len))
    } else {
        Some((format!("malformed: {candidate}"), len))
    }
}

fn print_context_block(hit: &HitWithContext, cand_len: usize) {
    for line in &hit.before {
        search::print_context_line("", line);
    }
    println!(
        "{} {}",
        "↪".cyan(),
        search::highlight_match(
            &hit.line_text,
            Span {
                offset: hit.line_offset,
                len: cand_len
            }
        )
    );
    for line in &hit.after {
        search::print_context_line("", line);
    }
}

struct SectionTracker {
    last_path: Option<String>,
    need_sep: bool,
}

impl SectionTracker {
    const fn new() -> Self {
        Self {
            last_path: None,
            need_sep: false,
        }
    }

    fn advance(&mut self, path: &str) {
        if self.need_sep {
            println!("{}", "--".cyan());
        }
        if self.last_path.as_deref() != Some(path) {
            println!("{}", path.magenta().bold().underline());
            self.last_path = Some(path.to_string());
        }
        self.need_sep = true;
    }
}

pub fn run(path: &Path) -> Result<usize> {
    let hits = search::search(path, BROAD_RE)?;
    let hits_ctx = search::add_context(hits, ContextWindow::symmetric(CONTEXT));
    let mut tracker = SectionTracker::new();
    let mut errors = 0;

    for hit in hits_ctx {
        let Some((label, cand_len)) = validate_hit(&hit.text) else {
            continue;
        };
        errors += 1;
        tracker.advance(&hit.path);
        print_context_block(&hit, cand_len);
        println!("{}", format!("error: {label}").red().bold());
    }

    Ok(errors)
}
