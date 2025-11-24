//! URL Detection and Opening Handler
//!
//! Provides URL detection in terminal output and browser opening capabilities.
//!
//! # Features
//! - Regex-based URL detection (http://, https://, www.)
//! - Security validation before opening
//! - Cross-platform browser opening
//! - Ctrl+Click support
//!
//! # Security
//! All URLs are validated to prevent shell injection attacks before being
//! passed to system commands.

use regex::Regex;
use std::sync::LazyLock;
use anyhow::{Result, Context};
use std::process::Command;

/// URL pattern matcher - more robust pattern that avoids common false positives
static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=]+|www\.[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=]+").unwrap()
});

/// Represents a detected URL in terminal output
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - position info used by consumers
pub struct DetectedUrl {
    pub url: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub line_number: usize,
}

/// URL Handler for detecting and opening URLs
pub struct UrlHandler {
    #[allow(dead_code)] // Public API field
    enabled: bool,
}

impl UrlHandler {
    #[allow(dead_code)] // Public API
    /// Create a new URL handler
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
    
    /// Detect URLs in text
    #[must_use]
    pub fn detect_urls(text: &str) -> Vec<DetectedUrl> {
        let mut urls = Vec::new();
        
        for (line_num, line) in text.lines().enumerate() {
            for m in URL_REGEX.find_iter(line) {
                let url = m.as_str().trim_end_matches(&['.', ',', ')', ']', '>', ';'][..]).to_string();
                urls.push(DetectedUrl {
                    url,
                    start_pos: m.start(),
                    end_pos: m.end(),
                    line_number: line_num,
                });
            }
        }
        
        urls
    }
    
    /// Validate URL before opening (basic validation)
    fn is_safe_url(url: &str) -> bool {
        // Basic sanity checks - reject URLs with shell metacharacters
        let dangerous_chars = &['<', '>', '|', '&', ';', '`', '$', '\\', '"', '\''];
        !url.chars().any(|c| dangerous_chars.contains(&c))
    }
    
    /// Open a URL in the default browser
    pub fn open_url(url: &str) -> Result<()> {
        let url = Self::normalize_url(url);
        
        // Validate URL before opening
        if !Self::is_safe_url(&url) {
            return Err(anyhow::anyhow!("URL contains potentially dangerous characters"));
        }
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/c", "start", "", &url])  // Empty string after start prevents command injection
                .spawn()
                .context("Failed to open URL")?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&url)
                .spawn()
                .context("Failed to open URL")?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()
                .context("Failed to open URL")?;
        }
        
        Ok(())
    }
    
    /// Normalize URL (add http:// if missing)
    fn normalize_url(url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else if url.starts_with("www.") {
            format!("http://{}", url)
        } else {
            url.to_string()
        }
    }
    
    /// Check if URL handler is enabled
    #[allow(dead_code)] // Public API
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable URL handler
    #[allow(dead_code)] // Public API
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_http_url() {
        let text = "Check out http://example.com for more info";
        let urls = UrlHandler::detect_urls(text);
        
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://example.com");
    }
    
    #[test]
    fn test_detect_https_url() {
        let text = "Visit https://github.com/user/repo";
        let urls = UrlHandler::detect_urls(text);
        
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://github.com/user/repo");
    }
    
    #[test]
    fn test_detect_www_url() {
        let text = "Go to www.example.com";
        let urls = UrlHandler::detect_urls(text);
        
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "www.example.com");
    }
    
    #[test]
    fn test_detect_multiple_urls() {
        let text = "Check http://example.com and https://github.com";
        let urls = UrlHandler::detect_urls(text);
        
        assert_eq!(urls.len(), 2);
    }
    
    #[test]
    fn test_no_urls() {
        let text = "This is just plain text";
        let urls = UrlHandler::detect_urls(text);
        
        assert_eq!(urls.len(), 0);
    }
    
    #[test]
    fn test_normalize_url() {
        assert_eq!(UrlHandler::normalize_url("http://example.com"), "http://example.com");
        assert_eq!(UrlHandler::normalize_url("https://example.com"), "https://example.com");
        assert_eq!(UrlHandler::normalize_url("www.example.com"), "http://www.example.com");
    }
    
    #[test]
    fn test_url_handler_enabled() {
        let mut handler = UrlHandler::new(true);
        assert!(handler.is_enabled());
        
        handler.set_enabled(false);
        assert!(!handler.is_enabled());
    }
}
