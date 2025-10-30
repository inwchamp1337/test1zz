use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use url::Url;
use crate::crawler::errors::{FileOperationError, CrawlerResult};
use log::{debug, error, info, trace, warn};

/// Manages file operations for saving Markdown content
pub struct FileManager {
    output_dir: PathBuf,
    filename_counter: HashMap<String, u32>,
}

impl FileManager {
    /// Creates a new FileManager with the specified output directory
    pub fn new(output_dir: &str) -> CrawlerResult<Self> {
        trace!("Creating FileManager with output directory: {}", output_dir);
        
        let output_path = PathBuf::from(output_dir);
        
        // Validate output directory path
        if output_dir.is_empty() {
            error!("Output directory path is empty");
            return Err(FileOperationError::InvalidPath("Empty path".to_string()).into());
        }

        // Create output directory if it doesn't exist
        if !output_path.exists() {
            info!("Creating output directory: {}", output_dir);
            fs::create_dir_all(&output_path)
                .map_err(|e| {
                    error!("Failed to create output directory '{}': {}", output_dir, e);
                    FileOperationError::DirectoryCreationFailed(e)
                })?;
        } else {
            debug!("Output directory already exists: {}", output_dir);
        }

        // Check if directory is writable
        if let Err(e) = fs::metadata(&output_path) {
            error!("Cannot access output directory '{}': {}", output_dir, e);
            return Err(FileOperationError::PermissionDenied(output_dir.to_string()).into());
        }
        
        info!("FileManager initialized successfully with directory: {}", output_dir);
        Ok(FileManager {
            output_dir: output_path,
            filename_counter: HashMap::new(),
        })
    }

    /// Saves Markdown content to a file based on the source URL
    pub fn save_markdown(&mut self, url: &str, content: &str) -> CrawlerResult<PathBuf> {
        trace!("Saving Markdown for URL: {} ({} bytes)", url, content.len());
        
        // Validate inputs
        if url.is_empty() {
            error!("URL is empty");
            return Err(FileOperationError::InvalidPath("Empty URL".to_string()).into());
        }
        
        if content.is_empty() {
            warn!("Content is empty for URL: {}", url);
        }

        let filename = self.generate_filename(url);
        debug!("Generated filename: {}", filename);
        
        let file_path = self.ensure_unique_filename(&self.output_dir.join(&filename));
        debug!("Final file path: {:?}", file_path);
        
        // Attempt to write file with retry logic
        self.write_file_with_retry(&file_path, content, 3)?;
        
        info!("Successfully saved Markdown file: {:?} ({} bytes)", file_path, content.len());
        Ok(file_path)
    }

    /// Write file with retry logic for transient errors
    fn write_file_with_retry(&self, file_path: &Path, content: &str, max_retries: usize) -> CrawlerResult<()> {
        let mut attempts = 0;
        
        while attempts < max_retries {
            match fs::write(file_path, content) {
                Ok(()) => {
                    if attempts > 0 {
                        info!("File write succeeded on attempt {}", attempts + 1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    
                    match e.kind() {
                        io::ErrorKind::PermissionDenied => {
                            error!("Permission denied writing to: {:?}", file_path);
                            return Err(FileOperationError::PermissionDenied(
                                file_path.to_string_lossy().to_string()
                            ).into());
                        }
                        io::ErrorKind::NotFound => {
                            error!("Directory not found for: {:?}", file_path);
                            return Err(FileOperationError::InvalidPath(
                                file_path.to_string_lossy().to_string()
                            ).into());
                        }
                        _ => {
                            if attempts < max_retries {
                                warn!("File write attempt {} failed: {}. Retrying...", attempts, e);
                                std::thread::sleep(std::time::Duration::from_millis(100 * attempts as u64));
                            } else {
                                error!("File write failed after {} attempts: {}", max_retries, e);
                                return Err(FileOperationError::FileWriteFailed(e).into());
                            }
                        }
                    }
                }
            }
        }
        
        unreachable!()
    }

    /// Generates a filename from a URL
    fn generate_filename(&self, url: &str) -> String {
        // Parse URL to extract meaningful parts
        if let Ok(parsed_url) = Url::parse(url) {
            let path = parsed_url.path();
            
            // Extract filename from path
            let filename = if path == "/" || path.is_empty() {
                // Use domain for root pages
                format!("{}_index", parsed_url.host_str().unwrap_or("unknown"))
            } else {
                // Use path segments for meaningful names
                let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                if segments.is_empty() {
                    format!("{}_index", parsed_url.host_str().unwrap_or("unknown"))
                } else {
                    segments.join("_")
                }
            };
            
            self.sanitize_filename(&filename)
        } else {
            // Fallback for invalid URLs
            self.sanitize_filename("unknown_page")
        }
    }

    /// Sanitizes filename to remove invalid characters
    fn sanitize_filename(&self, filename: &str) -> String {
        let mut sanitized = filename
            .chars()
            .map(|c| match c {
                // Replace invalid filesystem characters
                '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
                '/' | '\\' => '_',
                // Keep alphanumeric, underscore, hyphen, and dot
                c if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' => c,
                // Replace other characters with underscore
                _ => '_',
            })
            .collect::<String>();

        // Ensure filename doesn't start with dot or hyphen
        if sanitized.starts_with('.') || sanitized.starts_with('-') {
            sanitized = format!("page_{}", sanitized);
        }

        // Limit filename length (most filesystems support 255 chars)
        if sanitized.len() > 200 {
            sanitized.truncate(200);
        }

        // Ensure we have a valid filename
        if sanitized.is_empty() {
            sanitized = "page".to_string();
        }

        format!("{}.md", sanitized)
    }

    /// Ensures filename is unique by appending a counter if needed
    fn ensure_unique_filename(&mut self, base_path: &Path) -> PathBuf {
        let base_name = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");
        
        let extension = base_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("md");

        let mut final_path = base_path.to_path_buf();
        
        // Check if file already exists
        if final_path.exists() {
            let counter = self.filename_counter.entry(base_name.to_string()).or_insert(0);
            *counter += 1;
            
            let new_filename = format!("{}_{}.{}", base_name, counter, extension);
            final_path = base_path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(new_filename);
        }
        
        final_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output");
        
        let file_manager = FileManager::new(output_path.to_str().unwrap()).unwrap();
        
        assert!(output_path.exists());
        assert_eq!(file_manager.output_dir, output_path);
    }

    #[test]
    fn test_generate_filename_from_root_url() {
        let file_manager = FileManager::new("test").unwrap();
        let filename = file_manager.generate_filename("https://example.com/");
        
        assert_eq!(filename, "example.com_index.md");
    }

    #[test]
    fn test_generate_filename_from_path_url() {
        let file_manager = FileManager::new("test").unwrap();
        let filename = file_manager.generate_filename("https://example.com/docs/getting-started");
        
        assert_eq!(filename, "docs_getting-started.md");
    }

    #[test]
    fn test_sanitize_filename_removes_invalid_chars() {
        let file_manager = FileManager::new("test").unwrap();
        let sanitized = file_manager.sanitize_filename("file<>:\"name|?*");
        
        assert_eq!(sanitized, "file____name___.md");
    }

    #[test]
    fn test_sanitize_filename_handles_dots_and_hyphens() {
        let file_manager = FileManager::new("test").unwrap();
        let sanitized = file_manager.sanitize_filename(".hidden-file");
        
        assert_eq!(sanitized, "page_.hidden-file.md");
    }

    #[test]
    fn test_save_markdown_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut file_manager = FileManager::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        let content = "# Test Content\n\nThis is a test.";
        let file_path = file_manager.save_markdown("https://example.com/test", content).unwrap();
        
        assert!(file_path.exists());
        let saved_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved_content, content);
    }

    #[test]
    fn test_ensure_unique_filename_handles_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let mut file_manager = FileManager::new(temp_dir.path().to_str().unwrap()).unwrap();
        
        // Create first file
        let content1 = "First file content";
        let file_path1 = file_manager.save_markdown("https://example.com/test", content1).unwrap();
        
        // Create second file with same URL
        let content2 = "Second file content";
        let file_path2 = file_manager.save_markdown("https://example.com/test", content2).unwrap();
        
        // Files should have different names
        assert_ne!(file_path1, file_path2);
        assert!(file_path1.exists());
        assert!(file_path2.exists());
        
        // Verify content is different
        let saved_content1 = fs::read_to_string(&file_path1).unwrap();
        let saved_content2 = fs::read_to_string(&file_path2).unwrap();
        assert_eq!(saved_content1, content1);
        assert_eq!(saved_content2, content2);
    }
}