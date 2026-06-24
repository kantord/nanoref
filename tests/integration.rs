use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn nref() -> Command {
    Command::cargo_bin("nref").unwrap()
}

// --- mint ---

#[test]
fn mint_produces_correctly_formatted_marker() {
    let out = nref().arg("mint").output().unwrap();
    let marker = std::str::from_utf8(&out.stdout).unwrap().trim();
    assert!(marker.starts_with("nref-"), "must start with 'nref-'");
    assert_eq!(marker.len(), 28, "must be exactly 28 chars");
    assert!(
        marker[5..].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        "suffix must be nanoid-alphabet only"
    );
}

#[test]
fn minted_marker_passes_check() {
    let marker = mint_one();
    let dir = write_dir(&[("a.txt", &marker)]);
    nref().arg("check").arg(dir.path()).assert().success();
}

// --- check ---

#[test]
fn check_is_silent_on_clean_tree() {
    let dir = write_dir(&[("a.txt", "no markers here")]);
    nref().arg("check").arg(dir.path()).assert().success().stdout("");
}

#[test]
fn check_catches_bad_checksum() {
    let marker = mint_one();
    let corrupted = corrupt_check_char(&marker);
    let dir = write_dir(&[("a.txt", &corrupted)]);
    nref()
        .arg("check")
        .arg(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("bad checksum"));
}

#[test]
fn check_catches_malformed_length() {
    // Valid alphabet, wrong length — should be flagged as malformed
    let dir = write_dir(&[("a.txt", "nref-abcdefghijklmnopqr")]);
    nref()
        .arg("check")
        .arg(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("malformed"));
}

#[test]
fn check_exits_0_when_all_markers_valid() {
    let m1 = mint_one();
    let m2 = mint_one();
    let dir = write_dir(&[
        ("a.txt", &format!("ref: {}", m1)),
        ("b.txt", &format!("ref: {} and {}", m1, m2)),
    ]);
    nref().arg("check").arg(dir.path()).assert().success();
}

// --- ls ---

#[test]
fn ls_is_silent_on_empty_tree() {
    let dir = write_dir(&[("a.txt", "nothing here")]);
    nref().arg("ls").arg(dir.path()).assert().success().stdout("");
}

#[test]
fn ls_groups_shared_marker_across_files() {
    let marker = mint_one();
    let dir = write_dir(&[
        ("a.txt", &format!("see {}", marker)),
        ("b.txt", &format!("also {}", marker)),
    ]);
    nref()
        .arg("ls")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2 location(s)"));
}

#[test]
fn ls_counts_each_distinct_marker_separately() {
    let m1 = mint_one();
    let m2 = mint_one();
    let dir = write_dir(&[("a.txt", &format!("{} {}", m1, m2))]);
    let out = nref().arg("ls").arg(dir.path()).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&m1[..10]), "m1 should appear in output");
    assert!(stdout.contains(&m2[..10]), "m2 should appear in output");
}

// --- edge cases ---

#[test]
fn check_does_not_false_positive_on_marker_with_extension() {
    // A valid 28-char marker immediately followed by "-suffix" should not be
    // reported as malformed — check truncates the greedy broad match to MARKER_LEN.
    let marker = mint_one();
    let content = format!("{}-extension", marker);
    let dir = write_dir(&[("a.txt", &content)]);
    nref().arg("check").arg(dir.path()).assert().success();
}

#[test]
fn ls_finds_marker_when_followed_by_underscore() {
    // Marker ending in alphanumeric + immediately followed by _ breaks \b
    // 95%+ of nanoid markers end in alphanumeric — this affects nearly all of them
    let marker = mint_one();
    let content = format!("{}_tag", marker);
    let dir = write_dir(&[("a.txt", &content)]);
    nref()
        .arg("ls")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1 location(s)"));
}

#[test]
fn check_does_not_print_separator_between_different_files() {
    let bad = corrupt_check_char(&mint_one());
    let dir = write_dir(&[("a.txt", &bad), ("b.txt", &bad)]);
    let out = String::from_utf8(
        nref().arg("check").arg(dir.path()).output().unwrap().stdout,
    )
    .unwrap();
    // "--" should only appear between non-adjacent blocks in the same file,
    // not as a separator between different files
    let separators: Vec<&str> = out.lines().filter(|l| l.trim() == "--").collect();
    assert_eq!(separators.len(), 0, "no -- between different files, got:\n{}", out);
}

#[test]
fn check_does_not_duplicate_separator_for_adjacent_errors_in_same_file() {
    let bad1 = corrupt_check_char(&mint_one());
    let bad2 = corrupt_check_char(&mint_one());
    // Put them on adjacent lines — context windows overlap
    let content = format!("{}\n{}", bad1, bad2);
    let dir = write_dir(&[("a.txt", &content)]);
    let out = String::from_utf8(
        nref().arg("check").arg(dir.path()).output().unwrap().stdout,
    )
    .unwrap();
    let separators: Vec<&str> = out.lines().filter(|l| l.trim() == "--").collect();
    // Overlapping context: a "--" here would imply a gap that doesn't exist
    assert_eq!(separators.len(), 0, "no -- for overlapping contexts, got:\n{}", out);
}

#[test]
fn ls_context_is_empty_for_marker_on_first_line() {
    let marker = mint_one();
    let content = format!("{}\nline two\nline three", marker);
    let dir = write_dir(&[("a.txt", &content)]);
    let out = String::from_utf8(
        nref().arg("ls").arg(dir.path()).output().unwrap().stdout,
    )
    .unwrap();
    // No context line should appear before the match (it's line 1)
    assert!(!out.contains("0-"), "no context line numbered 0, got:\n{}", out);
}

#[test]
fn ls_context_is_empty_for_marker_on_last_line() {
    let marker = mint_one();
    let content = format!("line one\nline two\n{}", marker);
    let dir = write_dir(&[("a.txt", &content)]);
    let out = String::from_utf8(
        nref().arg("ls").arg(dir.path()).output().unwrap().stdout,
    )
    .unwrap();
    // No context line should appear after the match (it's the last line)
    assert!(!out.contains("4-"), "no phantom context line after last line, got:\n{}", out);
}

// --- helpers ---

fn mint_one() -> String {
    let out = nref().arg("mint").output().unwrap();
    std::str::from_utf8(&out.stdout).unwrap().trim().to_string()
}

fn write_dir(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (name, content) in files {
        fs::write(dir.path().join(name), content).unwrap();
    }
    dir
}

/// Flip the first check character to something different so the checksum is wrong.
fn corrupt_check_char(marker: &str) -> String {
    let mut chars: Vec<char> = marker.chars().collect();
    chars[5] = if chars[5] == 'A' { 'B' } else { 'A' };
    chars.into_iter().collect()
}
