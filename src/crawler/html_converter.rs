use crate::crawler::errors::{HtmlConversionError, CrawlerResult};
use log::{debug, error, info, trace, warn};

/// HTML to Markdown converter that processes HTML content and converts it to readable Markdown format
pub struct HtmlConverter;

// Keep the old ConversionError for backward compatibility, but deprecate it
#[deprecated(note = "Use HtmlConversionError from errors module instead")]
#[derive(Debug)]
pub enum ConversionError {
    ParseError(String),
    ProcessingError(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConversionError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for ConversionError {}

// Convert old error type to new error type
impl From<ConversionError> for HtmlConversionError {
    fn from(err: ConversionError) -> Self {
        match err {
            ConversionError::ParseError(msg) => HtmlConversionError::ParseError(msg),
            ConversionError::ProcessingError(msg) => HtmlConversionError::ProcessingError(msg),
        }
    }
}

impl HtmlConverter {
    /// Create a new HTML converter instance
    pub fn new() -> Self {
        Self
    }

    /// Convert HTML content to Markdown format
    /// Supports h1, h2, p, ul, li, ol, a, img, strong, em, br, blockquote tags
    pub fn convert_to_markdown(&self, html: &str) -> CrawlerResult<String> {
        trace!("Starting HTML to Markdown conversion ({} bytes)", html.len());
        
        if html.trim().is_empty() {
            warn!("HTML content is empty, returning empty string");
            return Err(HtmlConversionError::EmptyContent.into());
        }

        // Validate HTML structure
        if !self.validate_html_structure(html) {
            error!("Invalid HTML structure detected");
            return Err(HtmlConversionError::InvalidHtml("Malformed HTML structure".to_string()).into());
        }

        let mut content = html.to_string();
        debug!("Processing HTML content with {} characters", content.len());
        
        // Remove script and style tags completely
        content = self.remove_unwanted_tags(&content);
        
        // Convert HTML tags to Markdown in order of complexity
        content = self.convert_headings(&content);
        content = self.convert_blockquotes(&content);
        content = self.convert_lists(&content);
        content = self.convert_images(&content);
        content = self.convert_links(&content);
        content = self.convert_formatting(&content);
        content = self.convert_paragraphs(&content);
        content = self.convert_line_breaks(&content);
        
        // Clean up extra whitespace and normalize line endings
        content = self.clean_whitespace(&content);
        
        debug!("HTML conversion completed successfully ({} -> {} characters)", 
               html.len(), content.len());
        
        Ok(content)
    }

    /// Remove script, style, and other unwanted tags
    fn remove_unwanted_tags(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Remove script tags and their content
        while let Some(start) = result.find("<script") {
            if let Some(end) = result[start..].find("</script>") {
                let end_pos = start + end + 9; // length of "</script>"
                result.replace_range(start..end_pos, "");
            } else {
                break;
            }
        }
        
        // Remove style tags and their content
        while let Some(start) = result.find("<style") {
            if let Some(end) = result[start..].find("</style>") {
                let end_pos = start + end + 8; // length of "</style>"
                result.replace_range(start..end_pos, "");
            } else {
                break;
            }
        }
        
        result
    }

    /// Convert h1, h2 tags to Markdown headings
    fn convert_headings(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Convert h1 tags
        result = self.convert_tag_with_content(&result, "h1", "# ");
        
        // Convert h2 tags
        result = self.convert_tag_with_content(&result, "h2", "## ");
        
        result
    }

    /// Convert blockquote tags to Markdown blockquotes
    fn convert_blockquotes(&self, content: &str) -> String {
        self.convert_tag_with_content(content, "blockquote", "> ")
    }

    /// Convert ul, ol, li tags to Markdown lists
    fn convert_lists(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // First, convert list items
        result = self.convert_list_items(&result);
        
        // Then remove the list container tags
        result = self.remove_tag_pair(&result, "ul");
        result = self.remove_tag_pair(&result, "ol");
        
        result
    }

    /// Convert li tags to Markdown list items
    fn convert_list_items(&self, content: &str) -> String {
        let mut result = content.to_string();
        let mut pos = 0;
        
        while let Some(start) = result[pos..].find("<li") {
            let actual_start = pos + start;
            
            // Find the end of the opening tag
            if let Some(tag_end) = result[actual_start..].find('>') {
                let content_start = actual_start + tag_end + 1;
                
                // Find the closing tag
                if let Some(close_start) = result[content_start..].find("</li>") {
                    let content_end = content_start + close_start;
                    let li_content = result[content_start..content_end].trim();
                    
                    // Replace with Markdown list item
                    let markdown_item = format!("- {}\n", li_content);
                    result.replace_range(actual_start..content_end + 5, &markdown_item);
                    
                    pos = actual_start + markdown_item.len();
                } else {
                    pos = content_start;
                }
            } else {
                pos = actual_start + 1;
            }
        }
        
        result
    }

    /// Convert img tags to Markdown images
    fn convert_images(&self, content: &str) -> String {
        let mut result = content.to_string();
        let mut pos = 0;
        
        while let Some(start) = result[pos..].find("<img") {
            let actual_start = pos + start;
            
            // Find the end of the img tag
            if let Some(tag_end) = result[actual_start..].find('>') {
                let tag_content = &result[actual_start..actual_start + tag_end + 1];
                
                // Extract src and alt attributes
                let src = self.extract_attribute(tag_content, "src").unwrap_or_default();
                let alt = self.extract_attribute(tag_content, "alt").unwrap_or_default();
                
                // Create Markdown image
                let markdown_img = format!("![{}]({})", alt, src);
                
                result.replace_range(actual_start..actual_start + tag_end + 1, &markdown_img);
                pos = actual_start + markdown_img.len();
            } else {
                pos = actual_start + 1;
            }
        }
        
        result
    }

    /// Convert a tags to Markdown links
    fn convert_links(&self, content: &str) -> String {
        let mut result = content.to_string();
        let mut pos = 0;
        
        while let Some(start) = result[pos..].find("<a") {
            let actual_start = pos + start;
            
            // Find the end of the opening tag
            if let Some(tag_end) = result[actual_start..].find('>') {
                let tag_content = &result[actual_start..actual_start + tag_end + 1];
                let content_start = actual_start + tag_end + 1;
                
                // Find the closing tag
                if let Some(close_start) = result[content_start..].find("</a>") {
                    let content_end = content_start + close_start;
                    let link_text = result[content_start..content_end].trim();
                    
                    // Extract href attribute
                    let href = self.extract_attribute(tag_content, "href").unwrap_or_default();
                    
                    // Create Markdown link
                    let markdown_link = if href.is_empty() {
                        link_text.to_string()
                    } else {
                        format!("[{}]({})", link_text, href)
                    };
                    
                    result.replace_range(actual_start..content_end + 4, &markdown_link);
                    pos = actual_start + markdown_link.len();
                } else {
                    pos = content_start;
                }
            } else {
                pos = actual_start + 1;
            }
        }
        
        result
    }

    /// Convert strong, em tags to Markdown formatting
    fn convert_formatting(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Convert strong tags to **bold**
        result = self.convert_tag_with_content(&result, "strong", "**");
        result = self.convert_tag_with_content(&result, "b", "**");
        
        // Convert em tags to *italic*
        result = self.convert_tag_with_content(&result, "em", "*");
        result = self.convert_tag_with_content(&result, "i", "*");
        
        result
    }

    /// Convert p tags to paragraphs with double line breaks
    fn convert_paragraphs(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Replace opening p tags with nothing and closing p tags with double newlines
        result = result.replace("<p>", "");
        result = result.replace("</p>", "\n\n");
        
        // Handle self-closing p tags
        result = result.replace("<p/>", "\n\n");
        
        result
    }

    /// Convert br tags to line breaks
    fn convert_line_breaks(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        result = result.replace("<br>", "\n");
        result = result.replace("<br/>", "\n");
        result = result.replace("<br />", "\n");
        
        result
    }

    /// Helper function to convert a tag with content using prefix/suffix
    fn convert_tag_with_content(&self, content: &str, tag: &str, markdown_marker: &str) -> String {
        let mut result = content.to_string();
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        
        while let Some(start) = result.find(&open_tag) {
            if let Some(end_start) = result[start + open_tag.len()..].find(&close_tag) {
                let content_start = start + open_tag.len();
                let content_end = content_start + end_start;
                let tag_content = result[content_start..content_end].trim();
                
                let markdown_content = if tag == "h1" || tag == "h2" || tag == "blockquote" {
                    format!("{}{}\n", markdown_marker, tag_content)
                } else {
                    format!("{}{}{}", markdown_marker, tag_content, markdown_marker)
                };
                
                result.replace_range(start..content_end + close_tag.len(), &markdown_content);
            } else {
                break;
            }
        }
        
        result
    }

    /// Helper function to remove tag pairs completely
    fn remove_tag_pair(&self, content: &str, tag: &str) -> String {
        let mut result = content.to_string();
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        
        result = result.replace(&open_tag, "");
        result = result.replace(&close_tag, "");
        
        result
    }

    /// Extract attribute value from HTML tag
    fn extract_attribute(&self, tag_content: &str, attr_name: &str) -> Option<String> {
        let pattern = format!("{}=", attr_name);
        
        if let Some(start) = tag_content.find(&pattern) {
            let value_start = start + pattern.len();
            let remaining = &tag_content[value_start..];
            
            // Handle quoted attributes
            if remaining.starts_with('"') {
                if let Some(end) = remaining[1..].find('"') {
                    return Some(remaining[1..end + 1].to_string());
                }
            } else if remaining.starts_with('\'') {
                if let Some(end) = remaining[1..].find('\'') {
                    return Some(remaining[1..end + 1].to_string());
                }
            } else {
                // Handle unquoted attributes
                let end = remaining.find(' ').unwrap_or(remaining.len());
                return Some(remaining[..end].to_string());
            }
        }
        
        None
    }

    /// Clean up extra whitespace and normalize line endings
    fn clean_whitespace(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Remove HTML entities
        result = result.replace("&nbsp;", " ");
        result = result.replace("&amp;", "&");
        result = result.replace("&lt;", "<");
        result = result.replace("&gt;", ">");
        result = result.replace("&quot;", "\"");
        
        // Remove remaining HTML tags
        result = self.remove_remaining_html_tags(&result);
        
        // Normalize multiple spaces to single space
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }
        
        // Normalize multiple newlines (keep max 2 for paragraph separation)
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }
        
        // Trim leading and trailing whitespace
        result.trim().to_string()
    }

    /// Remove any remaining HTML tags
    fn remove_remaining_html_tags(&self, content: &str) -> String {
        let mut result = content.to_string();
        let mut pos = 0;
        
        while let Some(start) = result[pos..].find('<') {
            let actual_start = pos + start;
            
            if let Some(end) = result[actual_start..].find('>') {
                let tag_end = actual_start + end + 1;
                result.replace_range(actual_start..tag_end, "");
                pos = actual_start;
            } else {
                pos = actual_start + 1;
            }
        }
        
        result
    }

    /// Validate HTML structure for basic correctness
    fn validate_html_structure(&self, html: &str) -> bool {
        // Basic validation: check for balanced angle brackets
        let open_brackets = html.chars().filter(|&c| c == '<').count();
        let close_brackets = html.chars().filter(|&c| c == '>').count();
        
        if open_brackets != close_brackets {
            warn!("Unbalanced HTML brackets: {} open, {} close", open_brackets, close_brackets);
            return false;
        }

        // Check for extremely malformed HTML (no content between tags)
        if html.len() > 0 && html.chars().all(|c| c == '<' || c == '>' || c.is_whitespace()) {
            warn!("HTML appears to contain only tags and whitespace");
            return false;
        }

        true
    }

    /// Convert HTML with error recovery
    pub fn convert_to_markdown_with_recovery(&self, html: &str) -> CrawlerResult<String> {
        match self.convert_to_markdown(html) {
            Ok(markdown) => Ok(markdown),
            Err(e) => {
                warn!("Primary conversion failed, attempting recovery: {}", e);
                
                // Try to recover by cleaning the HTML first
                let cleaned_html = self.clean_malformed_html(html);
                match self.convert_to_markdown(&cleaned_html) {
                    Ok(markdown) => {
                        info!("HTML conversion recovered successfully after cleaning");
                        Ok(markdown)
                    }
                    Err(recovery_error) => {
                        error!("HTML conversion recovery failed: {}", recovery_error);
                        Err(recovery_error)
                    }
                }
            }
        }
    }

    /// Clean malformed HTML for recovery attempts
    fn clean_malformed_html(&self, html: &str) -> String {
        let mut cleaned = html.to_string();
        
        // Remove unclosed tags at the end
        while cleaned.ends_with('<') {
            cleaned.pop();
        }
        
        // Remove unopened tags at the beginning
        while cleaned.starts_with('>') {
            cleaned.remove(0);
        }
        
        // Fix common malformed patterns
        cleaned = cleaned.replace("<<", "<");
        cleaned = cleaned.replace(">>", ">");
        cleaned = cleaned.replace("<>", "");
        
        debug!("Cleaned malformed HTML: {} -> {} characters", html.len(), cleaned.len());
        cleaned
    }
}

impl Default for HtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_headings() {
        let converter = HtmlConverter::new();
        let html = "<h1>Title</h1><h2>Subtitle</h2>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("# Title"));
        assert!(result.contains("## Subtitle"));
    }

    #[test]
    fn test_convert_paragraphs() {
        let converter = HtmlConverter::new();
        let html = "<p>First paragraph</p><p>Second paragraph</p>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("First paragraph"));
        assert!(result.contains("Second paragraph"));
    }

    #[test]
    fn test_convert_lists() {
        let converter = HtmlConverter::new();
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 2"));
    }

    #[test]
    fn test_convert_links() {
        let converter = HtmlConverter::new();
        let html = r#"<a href="https://example.com">Example Link</a>"#;
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("[Example Link](https://example.com)"));
    }

    #[test]
    fn test_convert_images() {
        let converter = HtmlConverter::new();
        let html = r#"<img src="image.jpg" alt="Test Image">"#;
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("![Test Image](image.jpg)"));
    }

    #[test]
    fn test_convert_formatting() {
        let converter = HtmlConverter::new();
        let html = "<strong>Bold text</strong> and <em>italic text</em>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("**Bold text**"));
        assert!(result.contains("*italic text*"));
    }

    #[test]
    fn test_convert_blockquotes() {
        let converter = HtmlConverter::new();
        let html = "<blockquote>This is a quote</blockquote>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("> This is a quote"));
    }

    #[test]
    fn test_convert_line_breaks() {
        let converter = HtmlConverter::new();
        let html = "Line 1<br>Line 2<br/>Line 3";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(result.contains("Line 1\nLine 2\nLine 3"));
    }

    #[test]
    fn test_empty_html() {
        let converter = HtmlConverter::new();
        // Empty HTML should return an error as it's not valid HTML
        let result = converter.convert_to_markdown("");
        assert!(result.is_err(), "Empty HTML should return an error");
        
        // But empty content with valid HTML structure should work
        let empty_valid_html = "<html><body></body></html>";
        let result = converter.convert_to_markdown(empty_valid_html).unwrap();
        assert!(result.is_empty() || result.trim().is_empty(), "Empty valid HTML should return empty or whitespace-only content");
    }

    #[test]
    fn test_remove_script_tags() {
        let converter = HtmlConverter::new();
        let html = "<p>Content</p><script>alert('test');</script><p>More content</p>";
        let result = converter.convert_to_markdown(html).unwrap();
        assert!(!result.contains("alert"));
        assert!(result.contains("Content"));
        assert!(result.contains("More content"));
    }
}