use anyhow::Result;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io;
use std::path::Path;

pub struct Hit {
    pub text: String,
    pub path: String,
    pub line: u64,
    pub line_offset: usize,
}

pub struct HitWithContext {
    pub text: String,
    pub path: String,
    pub line: u64,
    pub line_text: String,
    pub line_offset: usize,
    pub before: Vec<(u64, String)>,
    pub after: Vec<(u64, String)>,
}

pub fn print_context_line(indent: &str, ln: u64, text: &str) {
    println!("{}{}{}", indent, format!("{}-", ln).green(), text);
}

struct Collector<'m> {
    matcher: &'m RegexMatcher,
    hits: Vec<(String, u64, usize)>,
}

impl Sink for Collector<'_> {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep_searcher::Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, io::Error> {
        let line_no = mat.line_number().unwrap_or(0);
        let bytes = mat.bytes();

        self.matcher
            .find_iter(bytes, |m| {
                if let Ok(text) = std::str::from_utf8(&bytes[m.start()..m.end()]) {
                    self.hits.push((text.to_string(), line_no, m.start()));
                }
                true
            })
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(true)
    }
}

pub fn search(path: &Path, pattern: &str) -> Result<Vec<Hit>> {
    let matcher = RegexMatcher::new(pattern)?;
    let mut searcher = SearcherBuilder::new().line_number(true).build();
    let mut hits = Vec::new();

    for result in WalkBuilder::new(path).build() {
        let entry = result?;
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_path = entry.path().display().to_string();
        let mut collector = Collector { matcher: &matcher, hits: Vec::new() };
        searcher.search_path(&matcher, entry.path(), &mut collector)?;

        for (text, line, line_offset) in collector.hits {
            hits.push(Hit { text, path: file_path.clone(), line, line_offset });
        }
    }

    Ok(hits)
}

pub fn add_context(hits: Vec<Hit>, before: usize, after: usize) -> Vec<HitWithContext> {
    let mut cache: HashMap<String, Vec<String>> = HashMap::new();

    hits.into_iter()
        .map(|hit| {
            let lines = cache.entry(hit.path.clone()).or_insert_with(|| {
                std::fs::read_to_string(&hit.path)
                    .map(|s| s.lines().map(String::from).collect())
                    .unwrap_or_default()
            });

            let idx = hit.line.saturating_sub(1) as usize;
            let start = idx.saturating_sub(before);
            let end = (idx + after + 1).min(lines.len());

            HitWithContext {
                line_text: lines.get(idx).cloned().unwrap_or_else(|| hit.text.clone()),
                before: (start..idx).map(|i| (i as u64 + 1, lines[i].clone())).collect(),
                after: ((idx + 1)..end).map(|i| (i as u64 + 1, lines[i].clone())).collect(),
                line_offset: hit.line_offset,
                text: hit.text,
                path: hit.path,
                line: hit.line,
            }
        })
        .collect()
}
