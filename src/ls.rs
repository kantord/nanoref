use crate::search::{self, ContextWindow, HitWithContext, Span};
use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;
use std::path::Path;

// No trailing \b — word boundaries fail when the marker is immediately followed
// by _ or alphanumeric (both are word chars), which would cause silent misses
// for markers in snake_case contexts.  Leading \b still prevents matching mid-word.
const MARKER_RE: &str = r"\bnref-[A-Za-z0-9_-]{23}";
const CONTEXT: usize = 3;

struct OccurrencePrinter {
    last_path: Option<String>,
    count: usize,
}

impl OccurrencePrinter {
    const fn new() -> Self {
        Self {
            last_path: None,
            count: 0,
        }
    }

    fn advance(&mut self, path: &str) {
        if self.count > 0 {
            println!("  {}", "--".cyan());
        }
        if self.last_path.as_deref() != Some(path) {
            println!("  {}", path.magenta().bold().underline());
            self.last_path = Some(path.to_string());
        }
        self.count += 1;
    }

    fn print(&mut self, hit: &HitWithContext) {
        self.advance(&hit.path);
        for line in &hit.before {
            search::print_context_line("  ", line);
        }
        println!(
            "  {} {}",
            "↪".cyan(),
            search::highlight_match(
                &hit.line_text,
                Span {
                    offset: hit.line_offset,
                    len: hit.text.len()
                }
            )
        );
        for line in &hit.after {
            search::print_context_line("  ", line);
        }
    }
}

fn print_marker_group(marker: &str, occurrences: &mut [HitWithContext]) {
    occurrences.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    println!("{}, {} location(s)", marker.bold(), occurrences.len());
    let mut printer = OccurrencePrinter::new();
    for hit in occurrences.iter() {
        printer.print(hit);
    }
    println!();
}

pub fn run(path: &Path) -> Result<()> {
    let hits = search::search(path, MARKER_RE)?;
    let hits_ctx = search::add_context(hits, ContextWindow::symmetric(CONTEXT));

    let mut index: BTreeMap<String, Vec<HitWithContext>> = BTreeMap::new();
    for hit in hits_ctx {
        index.entry(hit.text.clone()).or_default().push(hit);
    }

    for (marker, mut occurrences) in index {
        print_marker_group(&marker, &mut occurrences);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search;

    #[test]
    fn highlight_match_uses_given_offset_not_first_occurrence() {
        let marker = "nref-AABBCCDDEE00112233445";
        let line = format!("first {} second {}", marker, marker);
        let second_offset = line.rfind(marker).unwrap();
        let result = search::highlight_match(
            &line,
            Span {
                offset: second_offset,
                len: marker.len(),
            },
        );
        assert!(result.starts_with(&line[..second_offset]));
        assert!(result.ends_with(&line[second_offset + marker.len()..]));
    }
}
