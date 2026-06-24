use anyhow::Result;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::{SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io;
use std::ops::Range;
use std::path::Path;

pub struct Hit {
    pub text: String,
    pub path: String,
    pub line: u64,
    pub line_offset: usize,
}

pub struct ContextLine {
    pub number: u64,
    pub text: String,
}

pub struct HitWithContext {
    pub text: String,
    pub path: String,
    pub line: u64,
    pub line_text: String,
    pub line_offset: usize,
    pub before: Vec<ContextLine>,
    pub after: Vec<ContextLine>,
}

#[derive(Copy, Clone)]
pub struct ContextWindow {
    pub before: usize,
    pub after: usize,
}

impl ContextWindow {
    pub const fn symmetric(n: usize) -> Self {
        Self {
            before: n,
            after: n,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
}

pub fn print_context_line(indent: &str, line: &ContextLine) {
    println!(
        "{indent}{}{}",
        format!("{}-", line.number).green(),
        line.text
    );
}

pub fn highlight_match(line: &str, span: Span) -> String {
    let end = span.offset + span.len;
    format!(
        "{}{}{}",
        &line[..span.offset],
        (&line[span.offset..end]).red().bold(),
        &line[end..]
    )
}

struct Collector<'m> {
    matcher: &'m RegexMatcher,
    hits: Vec<(String, u64, usize)>,
}

impl<'m> Collector<'m> {
    const fn new(matcher: &'m RegexMatcher) -> Self {
        Self {
            matcher,
            hits: Vec::new(),
        }
    }

    fn into_hits(self, file_path: String) -> Vec<Hit> {
        self.hits
            .into_iter()
            .map(move |(text, line, line_offset)| Hit {
                text,
                path: file_path.clone(),
                line,
                line_offset,
            })
            .collect()
    }
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
            .map_err(|e| io::Error::other(e.to_string()))?;

        Ok(true)
    }
}

pub fn search(path: &Path, pattern: &str) -> Result<Vec<Hit>> {
    let matcher = RegexMatcher::new(pattern)?;
    let mut searcher = SearcherBuilder::new().line_number(true).build();
    let mut hits = Vec::new();

    for result in WalkBuilder::new(path).build() {
        let entry = result?;
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let file_path = entry.path().display().to_string();
        let mut collector = Collector::new(&matcher);
        searcher.search_path(&matcher, entry.path(), &mut collector)?;
        hits.extend(collector.into_hits(file_path));
    }

    Ok(hits)
}

fn build_context_lines(lines: &[String], range: Range<usize>) -> Vec<ContextLine> {
    range
        .map(|i| ContextLine {
            number: i as u64 + 1,
            text: lines[i].clone(),
        })
        .collect()
}

struct ContextResolver {
    cache: HashMap<String, Vec<String>>,
    window: ContextWindow,
}

impl ContextResolver {
    fn new(window: ContextWindow) -> Self {
        Self {
            cache: HashMap::new(),
            window,
        }
    }

    fn resolve(&mut self, hit: Hit) -> HitWithContext {
        let lines = self.cache.entry(hit.path.clone()).or_insert_with(|| {
            std::fs::read_to_string(&hit.path)
                .map(|s| s.lines().map(String::from).collect())
                .unwrap_or_default()
        });
        let idx = usize::try_from(hit.line.saturating_sub(1)).unwrap_or(0);
        let start = idx.saturating_sub(self.window.before);
        let end = (idx + self.window.after + 1).min(lines.len());
        HitWithContext {
            line_text: lines.get(idx).cloned().unwrap_or_else(|| hit.text.clone()),
            before: build_context_lines(lines, start..idx),
            after: build_context_lines(lines, (idx + 1)..end),
            line_offset: hit.line_offset,
            text: hit.text,
            path: hit.path,
            line: hit.line,
        }
    }
}

pub fn add_context(hits: Vec<Hit>, window: ContextWindow) -> Vec<HitWithContext> {
    let mut resolver = ContextResolver::new(window);
    hits.into_iter().map(|hit| resolver.resolve(hit)).collect()
}
