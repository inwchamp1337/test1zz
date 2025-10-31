

pub fn html_to_markdown(url: &str, html: &str) -> String {
    let mut output = String::new();
    
    // Remove script and style tags completely
    let html = remove_tags(html, &["script", "style", "noscript"]);
    
    // Convert common tags to markdown
    let html = convert_headings(&html);
    let html = convert_links(&html);
    let html = convert_images(&html);
    let html = convert_strong(&html);
    let html = convert_em(&html);
    let html = convert_lists(&html);
    let html = convert_blockquotes(&html);
    let html = convert_code(&html);
    
    // Remove remaining HTML tags
    let text = strip_html_tags(&html);
    
    // Clean up whitespace
    let text = clean_whitespace(&text);
    
    if text.trim().is_empty() {
        format!("# {}\n\nNo content extracted.\n", url)
    } else {
        text
    }
}

fn remove_tags(html: &str, tags: &[&str]) -> String {
    let mut result = html.to_string();
    for tag in tags {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        
        while let Some(start) = result.find(&open) {
            if let Some(end_pos) = result[start..].find(&close) {
                result.replace_range(start..start + end_pos + close.len(), "");
            } else {
                break;
            }
        }
    }
    result
}

fn convert_headings(html: &str) -> String {
    let mut result = html.to_string();
    for level in 1..=6 {
        let open = format!("<h{}", level);
        let close = format!("</h{}>", level);
        let marker = "#".repeat(level);
        
        while let Some(start) = result.find(&open) {
            if let Some(tag_end) = result[start..].find('>') {
                let content_start = start + tag_end + 1;
                if let Some(close_start) = result[content_start..].find(&close) {
                    let content = &result[content_start..content_start + close_start];
                    let markdown = format!("\n\n{} {}\n\n", marker, strip_html_tags(content).trim());
                    result.replace_range(start..content_start + close_start + close.len(), &markdown);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    result
}

fn convert_links(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find("<a ") {
        if let Some(href_start) = result[start..].find("href=\"") {
            let href_pos = start + href_start + 6;
            if let Some(href_end) = result[href_pos..].find('"') {
                let href = &result[href_pos..href_pos + href_end];
                if let Some(tag_end) = result[start..].find('>') {
                    let content_start = start + tag_end + 1;
                    if let Some(close) = result[content_start..].find("</a>") {
                        let text = &result[content_start..content_start + close];
                        let markdown = format!("[{}]({})", strip_html_tags(text).trim(), href);
                        result.replace_range(start..content_start + close + 4, &markdown);
                        continue;
                    }
                }
            }
        }
        break;
    }
    result
}

fn convert_images(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find("<img ") {
        if let Some(end) = result[start..].find('>') {
            let tag = &result[start..start + end + 1];
            let src = extract_attr(tag, "src");
            let alt = extract_attr(tag, "alt");
            let markdown = format!("\n\n![{}]({})\n\n", alt, src);
            result.replace_range(start..start + end + 1, &markdown);
        } else {
            break;
        }
    }
    result
}

fn convert_strong(html: &str) -> String {
    replace_tag_pair(html, "<strong>", "</strong>", "**")
        .replace("<b>", "**")
        .replace("</b>", "**")
}

fn convert_em(html: &str) -> String {
    replace_tag_pair(html, "<em>", "</em>", "*")
        .replace("<i>", "*")
        .replace("</i>", "*")
}

fn convert_lists(html: &str) -> String {
    let mut result = html.to_string();
    
    // Unordered lists
    while let Some(start) = result.find("<ul>") {
        if let Some(end) = result[start..].find("</ul>") {
            let content = &result[start + 4..start + end];
            let items = convert_list_items(content, "- ");
            result.replace_range(start..start + end + 5, &format!("\n\n{}\n\n", items));
        } else {
            break;
        }
    }
    
    // Ordered lists
    let mut ol_index = 1;
    while let Some(start) = result.find("<ol>") {
        if let Some(end) = result[start..].find("</ol>") {
            let content = &result[start + 4..start + end];
            let items = convert_list_items_ordered(content, &mut ol_index);
            result.replace_range(start..start + end + 5, &format!("\n\n{}\n\n", items));
        } else {
            break;
        }
    }
    
    result
}

fn convert_list_items(html: &str, prefix: &str) -> String {
    let mut result = String::new();
    let mut remaining = html;
    
    while let Some(start) = remaining.find("<li>") {
        if let Some(end) = remaining[start..].find("</li>") {
            let content = &remaining[start + 4..start + end];
            result.push_str(&format!("{}{}\n", prefix, strip_html_tags(content).trim()));
            remaining = &remaining[start + end + 5..];
        } else {
            break;
        }
    }
    
    result
}

fn convert_list_items_ordered(html: &str, start_index: &mut usize) -> String {
    let mut result = String::new();
    let mut remaining = html;
    
    while let Some(start) = remaining.find("<li>") {
        if let Some(end) = remaining[start..].find("</li>") {
            let content = &remaining[start + 4..start + end];
            result.push_str(&format!("{}. {}\n", start_index, strip_html_tags(content).trim()));
            *start_index += 1;
            remaining = &remaining[start + end + 5..];
        } else {
            break;
        }
    }
    
    result
}

fn convert_blockquotes(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find("<blockquote>") {
        if let Some(end) = result[start..].find("</blockquote>") {
            let content = &result[start + 12..start + end];
            let lines: Vec<_> = strip_html_tags(content)
                .lines()
                .map(|l| format!("> {}", l.trim()))
                .collect();
            result.replace_range(start..start + end + 13, &format!("\n\n{}\n\n", lines.join("\n")));
        } else {
            break;
        }
    }
    result
}

fn convert_code(html: &str) -> String {
    replace_tag_pair(html, "<code>", "</code>", "`")
}

fn replace_tag_pair(html: &str, open: &str, close: &str, markdown: &str) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find(open) {
        if let Some(end) = result[start..].find(close) {
            let content = &result[start + open.len()..start + end];
            let replacement = format!("{}{}{}", markdown, content, markdown);
            result.replace_range(start..start + end + close.len(), &replacement);
        } else {
            break;
        }
    }
    result
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    result
}

fn clean_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut prev_newline = false;
    
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_newline {
                result.push('\n');
                prev_newline = true;
            }
        } else {
            result.push_str(trimmed);
            result.push('\n');
            prev_newline = false;
        }
    }
    
    result.trim().to_string()
}

fn extract_attr(tag: &str, attr: &str) -> String {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = tag[value_start..].find('"') {
            return tag[value_start..value_start + end].to_string();
        }
    }
    String::new()
}