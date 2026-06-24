use crate::search;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::collections::BTreeMap;
use std::path::Path;

// No trailing \b — word boundaries fail when the marker is immediately followed
// by _ or alphanumeric (both are word chars), which would cause silent misses
// for markers in snake_case contexts.  Leading \b still prevents matching mid-word.
const MARKER_RE: &str = r"\bnref-[A-Za-z0-9_-]{23}";
const CONTEXT: usize = 3;

fn highlight_match(line: &str, offset: usize, len: usize) -> String {
    let end = offset + len;
    format!(
        "{}{}{}",
        &line[..offset],
        (&line[offset..end]).red().bold(),
        &line[end..]
    )
}

pub fn run(path: &Path) -> Result<()> {
    let hits = search::search(path, MARKER_RE)?;
    let hits_ctx = search::add_context(hits, CONTEXT, CONTEXT);

    let mut index: BTreeMap<String, Vec<search::HitWithContext>> = BTreeMap::new();
    for hit in hits_ctx {
        index.entry(hit.text.clone()).or_default().push(hit);
    }

    for (marker, mut occurrences) in index {
        occurrences.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
        let n = occurrences.len();
        println!("{}, {} location(s)", marker.bold(), n);

        let mut last_path: Option<String> = None;
        for (i, hit) in occurrences.iter().enumerate() {
            if i > 0 {
                println!("  {}", "--".cyan());
            }
            if last_path.as_deref() != Some(&hit.path) {
                println!("  {}", hit.path.magenta().bold().underline());
                last_path = Some(hit.path.clone());
            }
            for (ln, text) in &hit.before {
                search::print_context_line("  ", *ln, text);
            }
            println!(
                "  {} {}",
                "↪".cyan(),
                highlight_match(&hit.line_text, hit.line_offset, hit.text.len())
            );
            for (ln, text) in &hit.after {
                search::print_context_line("  ", *ln, text);
            }
        }
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_match_uses_given_offset_not_first_occurrence() {
        let marker = "nref-AABBCCDDEE00112233445";
        let line = format!("first {} second {}", marker, marker);
        let second_offset = line.rfind(marker).unwrap();
        let result = highlight_match(&line, second_offset, marker.len());
        // Everything before the second occurrence must be unchanged.
        assert!(result.starts_with(&line[..second_offset]));
        // Everything after the second occurrence must be unchanged.
        assert!(result.ends_with(&line[second_offset + marker.len()..]));
    }
}
