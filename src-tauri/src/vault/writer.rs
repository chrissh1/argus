//! Atomic vault writer. All writes go: temp file → fsync → rename.

use crate::{ArgusError, ArgusResult};
use chrono::Local;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write_new_note(
    vault_root: &Path,
    relative_path: &Path,
    title: &str,
    body: &str,
    tags: &[String],
    session_id: &str,
) -> ArgusResult<PathBuf> {
    let full = vault_root.join(relative_path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let frontmatter = build_frontmatter(title, tags, session_id);
    let final_text = format!("{frontmatter}\n\n# {title}\n\n{body}\n");
    atomic_write(&full, final_text.as_bytes())?;
    Ok(full)
}

pub fn append_to_note(
    vault_root: &Path,
    relative_path: &Path,
    section_heading: Option<&str>,
    new_text: &str,
) -> ArgusResult<PathBuf> {
    let full = vault_root.join(relative_path);
    let existing = std::fs::read_to_string(&full).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            ArgusError::NotFound(format!("vault note missing: {}", full.display()))
        }
        _ => e.into(),
    })?;

    let updated = match section_heading {
        Some(h) => insert_under_heading(&existing, h, new_text),
        None => format!("{}\n\n{}\n", existing.trim_end(), new_text.trim_end()),
    };

    atomic_write(&full, updated.as_bytes())?;
    Ok(full)
}

fn insert_under_heading(existing: &str, heading: &str, new_text: &str) -> String {
    // Find a line that begins with `## heading` or `### heading` (case-insensitive
    // on the heading text). Insert `new_text` at the end of that section.
    let target = heading.trim().to_lowercase();
    let lines: Vec<&str> = existing.lines().collect();
    let mut start: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let depth = trimmed.chars().take_while(|c| *c == '#').count();
        if depth == 0 {
            continue;
        }
        let text = trimmed.trim_start_matches('#').trim();
        if text.to_lowercase() == target {
            start = Some(i);
            break;
        }
    }
    let Some(start) = start else {
        // Heading not found — append at EOF.
        return format!("{}\n\n## {}\n\n{}\n", existing.trim_end(), heading, new_text.trim_end());
    };
    // Find end of this section: next heading of same-or-shallower depth.
    let start_depth = lines[start]
        .trim_start()
        .chars()
        .take_while(|c| *c == '#')
        .count();
    let mut end = lines.len();
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        let d = line.trim_start().chars().take_while(|c| *c == '#').count();
        if d > 0 && d <= start_depth {
            end = i;
            break;
        }
    }
    let mut buf = String::new();
    buf.push_str(&lines[..end].join("\n"));
    buf.push('\n');
    if !lines[end - 1].trim().is_empty() {
        buf.push('\n');
    }
    buf.push_str(new_text.trim_end());
    buf.push('\n');
    if end < lines.len() {
        buf.push('\n');
        buf.push_str(&lines[end..].join("\n"));
        buf.push('\n');
    }
    buf
}

fn build_frontmatter(title: &str, tags: &[String], session_id: &str) -> String {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let tag_list = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", tags.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", "))
    };
    format!(
        "---\ntitle: \"{title}\"\ndate: {date}\nsession_id: {session_id}\ntags: {tag_list}\nargus: true\n---"
    )
}

fn atomic_write(path: &Path, bytes: &[u8]) -> ArgusResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| ArgusError::Other(format!("no parent for {}", path.display())))?;
    std::fs::create_dir_all(parent)?;
    let mut tmp = tempfile_in(parent)?;
    tmp.file.write_all(bytes)?;
    tmp.file.sync_all()?;
    std::fs::rename(&tmp.path, path)?;
    Ok(())
}

struct TempHandle {
    path: PathBuf,
    file: std::fs::File,
}

fn tempfile_in(dir: &Path) -> ArgusResult<TempHandle> {
    let uniq = uuid::Uuid::new_v4().simple().to_string();
    let path = dir.join(format!(".argus-tmp-{uniq}"));
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    Ok(TempHandle { path, file })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_under_heading_appends_inside_section() {
        let doc = "# Title\n\n## Auth\n\nold content\n\n## Other\n\nelsewhere\n";
        let out = insert_under_heading(doc, "Auth", "new line");
        assert!(out.contains("old content\n\nnew line"));
        assert!(out.contains("## Other"));
    }

    #[test]
    fn insert_under_heading_missing_appends_at_end() {
        let doc = "# Title\n\nbody only\n";
        let out = insert_under_heading(doc, "New", "content");
        assert!(out.contains("## New\n\ncontent"));
    }
}
