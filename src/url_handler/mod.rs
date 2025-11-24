use regex::Regex;
use once_cell::sync::Lazy;
use anyhow::Result;
use std::process::Command;

/// URL pattern matcher
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s<>]+|www\.[^\s<>]+").unwrap()
});

/// Represents a detected URL in terminal output
#[derive(Debug, Clone)]
pub struct DetectedUrl {
    pub url: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub line_number: usize,
}

/// URL Handler for detecting and opening URLs
pub struct UrlHandler {
    enabled: bool,
}

impl UrlHandler {
    /// Create a new URL handler
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
    
    /// Detect URLs in text
    pub fn detect_urls(text: &str) -> Vec<DetectedUrl> {
        let mut urls = Vec::new();
        
        for (line_num, line) in text.lines().enumerate() {
            for m in URL_REGEX.find_iter(line) {
                let url = m.as_str().to_string();
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
    
    /// Open a URL in the default browser
    pub fn open_url(url: &str) -> Result<()> {
        let url = Self::normalize_url(url);
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/c", "start", &url])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&url)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()?;
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
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable or disable URL handler
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
