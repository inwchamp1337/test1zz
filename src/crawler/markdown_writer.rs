use spider::url::Url;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn write_markdown_file(url: &str, markdown: &str) -> Result<PathBuf, Box<dyn Error>> {
    let slug = slug_from_url(url);
    let mut path = PathBuf::from("output");
    fs::create_dir_all(&path)?;
    path.push(slug);
    path.set_extension("md");
    fs::write(&path, markdown)?;
    Ok(path)
}

fn slug_from_url(url: &str) -> String {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(mut segments) = parsed
            .path_segments()
            .map(|segments| segments.filter(|s| !s.is_empty()).map(sanitize_segment).collect::<Vec<_>>())
        {
            if let Some(seg) = segments.pop() {
                if !seg.is_empty() {
                    return seg;
                }
            }
        }
        return "index".into();
    }
    sanitize_segment(url)
}

fn sanitize_segment(segment: &str) -> String {
    let mut slug = String::new();
    for ch in segment.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_whitespace() || ch == '-' || ch == '_' {
            if !slug.ends_with('-') {
                slug.push('-');
            }
        }
    }
    slug.trim_matches('-').to_string()
}