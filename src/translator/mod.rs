//! Cross-platform command translation module
//!
//! Provides automatic translation between Linux and Windows commands to enable
//! seamless command-line usage across different operating systems.
//!
//! # Features
//! - 50+ common command mappings (ls↔dir, cat↔type, etc.)
//! - Comprehensive argument and flag translation
//! - Zero-copy design with static maps
//! - Configurable enable/disable
//!
//! # Supported Commands
//!
//! ## File System Commands
//! - `ls`↔`dir`: List directory contents with flag translation
//! - `cat`↔`type`: Display file contents
//! - `cp`↔`copy`/`xcopy`: Copy files with recursive support
//! - `mv`↔`move`: Move/rename files
//! - `rm`↔`del`: Remove files with flag translation
//! - `mkdir`↔`md`: Create directories
//! - `rmdir`↔`rd`: Remove directories
//! - `touch`↔`type nul >`: Create empty files
//! - `ln`↔`mklink`: Create symbolic links
//!
//! ## Text Processing Commands
//! - `grep`↔`findstr`: Search text patterns with flag translation
//! - `head`↔`Get-Content -Head`: Display first lines
//! - `tail`↔`Get-Content -Tail`: Display last lines
//! - `wc`↔`find /c`: Count lines/words/chars
//! - `sort`↔`sort`: Sort lines (with flag translation)
//! - `uniq`↔`Get-Unique`: Filter duplicate lines
//! - `cut`↔`ForEach-Object`: Extract columns
//! - `sed`↔`-replace`: Stream editor
//! - `awk`↔`ForEach-Object`: Pattern processing
//! - `tr`↔`-replace`: Translate characters
//! - `diff`↔`fc`: Compare files
//!
//! ## System Commands
//! - `pwd`↔`cd`: Print working directory
//! - `clear`↔`cls`: Clear screen
//! - `which`↔`where`: Locate command
//! - `whoami`↔`whoami`: Display current user
//! - `hostname`↔`hostname`: Display system hostname
//! - `env`↔`set`: Display environment variables
//! - `export`↔`set`: Set environment variables
//! - `uname`↔`ver`: Display system information
//! - `uptime`↔`net statistics`: Display system uptime
//! - `date`↔`date /t`: Display current date
//!
//! ## Process Commands
//! - `ps`↔`tasklist`: List processes
//! - `kill`↔`taskkill`: Terminate process
//! - `killall`↔`taskkill /IM`: Kill all processes by name
//!
//! ## Network Commands
//! - `ping`↔`ping`: Ping host (flag translation)
//! - `curl`↔`Invoke-WebRequest`: HTTP requests
//! - `wget`↔`Invoke-WebRequest`: Download files
//! - `ifconfig`/`ip`↔`ipconfig`: Network configuration
//! - `netstat`↔`netstat`: Network statistics
//! - `traceroute`↔`tracert`: Trace route
//! - `nslookup`↔`nslookup`: DNS lookup
//!
//! ## Disk Commands
//! - `df`↔`Get-PSDrive`: Display disk space
//! - `du`↔`dir /S`: Display disk usage
//! - `mount`↔`mountvol`: Mount filesystems
//!
//! ## Archive Commands
//! - `tar`↔`tar`: Archive files (Windows 10+ has tar)
//! - `zip`↔`Compress-Archive`: Create zip archives
//! - `unzip`↔`Expand-Archive`: Extract zip archives
//! - `gzip`↔`Compress-Archive`: Gzip compression
//!
//! ## Permission Commands
//! - `chmod`↔`icacls`: Change file permissions
//! - `chown`↔`icacls`: Change file owner
//!
//! ## Viewing Commands
//! - `less`↔`more`: Page through file
//! - `more`↔`more`: Page through file
//!
//! # Examples
//! ```
//! use furnace::translator::CommandTranslator;
//!
//! let translator = CommandTranslator::new(true);
//! let result = translator.translate("ls -la /home");
//! if result.translated {
//!     println!("Translated to: {}", result.final_command);
//! }
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;

/// Command translator for cross-platform command compatibility
/// Translates Linux commands to Windows equivalents and vice versa
#[derive(Debug, Clone)]
pub struct CommandTranslator {
    enabled: bool,
    current_os: OsType,
    // Use references to static maps instead of cloning
    _phantom: std::marker::PhantomData<()>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommandMapping {
    pub target_cmd: &'static str,
    pub description: &'static str,
    pub arg_translator: fn(&str) -> String,
}

#[derive(Debug)]
#[allow(dead_code)] // Public API - description may be used by consumers
pub struct TranslationResult {
    pub translated: bool,
    pub original_command: String,
    pub final_command: String,
    pub description: String,
    /// Errors encountered during translation (non-fatal)
    pub errors: Vec<TranslationError>,
    /// Whether the command contains pipelines
    pub has_pipeline: bool,
}

/// Errors that can occur during command translation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API - all variants available for consumers
pub enum TranslationError {
    /// Command not found in translation map
    UnknownCommand(String),
    /// Invalid syntax in command
    InvalidSyntax(String),
    /// Unsupported pipeline operator for translation
    UnsupportedOperator(String),
    /// Partial translation - some parts could not be translated
    PartialTranslation(String),
}

impl std::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: {cmd}"),
            Self::InvalidSyntax(msg) => write!(f, "Invalid syntax: {msg}"),
            Self::UnsupportedOperator(op) => write!(f, "Unsupported operator: {op}"),
            Self::PartialTranslation(msg) => write!(f, "Partial translation: {msg}"),
        }
    }
}

impl std::error::Error for TranslationError {}

/// Pipeline operators supported by the translator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelineOperator {
    /// Pipe operator `|`
    Pipe,
    /// Output redirect `>`
    RedirectOut,
    /// Output append `>>`
    RedirectAppend,
    /// Input redirect `<`
    RedirectIn,
    /// Command chain - run next if success `&&`
    And,
    /// Command chain - run next if failure `||`
    Or,
    /// Command separator `;`
    Semicolon,
}

impl PipelineOperator {
    /// Parse a pipeline operator from a string
    #[allow(dead_code)] // Used in tests
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "|" => Some(Self::Pipe),
            ">" => Some(Self::RedirectOut),
            ">>" => Some(Self::RedirectAppend),
            "<" => Some(Self::RedirectIn),
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            ";" => Some(Self::Semicolon),
            _ => None,
        }
    }

    /// Convert to string representation
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pipe => "|",
            Self::RedirectOut => ">",
            Self::RedirectAppend => ">>",
            Self::RedirectIn => "<",
            Self::And => "&&",
            Self::Or => "||",
            Self::Semicolon => ";",
        }
    }

    /// Translate operator between Linux and Windows
    #[allow(dead_code)]
    fn translate(&self, _target_os: OsType) -> &'static str {
        // Most operators are the same across platforms
        // PowerShell and cmd.exe support |, >, >>, <, &&, ||, ;
        self.as_str()
    }
}

/// A segment of a pipeline (a single command between operators)
#[derive(Debug, Clone)]
struct PipelineSegment {
    command: String,
    operator: Option<PipelineOperator>,
}

impl Default for CommandMapping {
    fn default() -> Self {
        Self {
            target_cmd: "",
            description: "",
            arg_translator: |args| args.to_string(),
        }
    }
}

// ============================================================================
// Argument Translators for Linux -> Windows
// ============================================================================

/// Returns arguments unchanged (identity function)
#[inline]
fn identity_args(args: &str) -> String {
    args.to_string()
}

/// Helper function to check if a flag is present (handles combined flags like -rf)
#[inline]
fn has_flag(args: &str, short: char, long: Option<&str>) -> bool {
    for part in args.split_whitespace() {
        if part.starts_with("--") {
            if let Some(l) = long {
                if part.strip_prefix("--") == Some(l) {
                    return true;
                }
            }
        } else if part.starts_with('-') && !part.starts_with("--") {
            // Handle combined short flags like -rf, -la, etc.
            if part.chars().skip(1).any(|c| c == short) {
                return true;
            }
        }
    }
    false
}

/// Helper function to extract non-flag arguments (paths, filenames, etc.) for Linux commands
/// Note: Paths can start with / on Linux (absolute paths)
#[inline]
fn extract_paths(args: &str) -> Vec<&str> {
    args.split_whitespace()
        .filter(|part| {
            // Filter out flags (start with -)
            // But allow paths starting with / (Linux absolute paths)
            !part.starts_with('-')
        })
        .collect()
}

/// Helper function to extract non-flag arguments for Windows commands
#[inline]
fn extract_paths_windows(args: &str) -> Vec<&str> {
    args.split_whitespace()
        .filter(|part| {
            // Keep paths that start with drive letters (C:\) or relative paths
            // Filter out Windows flags that start with /
            if part.starts_with('/') && part.len() <= 3 {
                // Likely a flag like /S, /A, /F
                false
            } else {
                !part.starts_with('-')
            }
        })
        .collect()
}

/// Helper to get flag value like -n 10 returns Some("10")
fn get_flag_value<'a>(args: &'a str, short: char, long: Option<&str>) -> Option<&'a str> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.starts_with("--") {
            if let Some(l) = long {
                if part.strip_prefix("--") == Some(l) {
                    return parts.get(i + 1).copied();
                }
                // Handle --flag=value format
                if let Some(rest) = part.strip_prefix("--") {
                    if let Some((key, value)) = rest.split_once('=') {
                        if key == l {
                            return Some(value);
                        }
                    }
                }
            }
        } else if part.starts_with('-') && !part.starts_with("--") {
            // Handle -n10 format (flag directly followed by value)
            if part.len() == 2 && part.chars().nth(1) == Some(short) {
                return parts.get(i + 1).copied();
            }
            // Handle -n10 (no space)
            if part.starts_with(&format!("-{short}")) && part.len() > 2 {
                return Some(&part[2..]);
            }
        }
    }
    None
}

/// Translates ls flags to Windows dir equivalents
/// Supported flags:
/// - `-a`, `--all` -> `/A` (show hidden files)
/// - `-l`, `--long` -> (detailed format, default in dir)
/// - `-R`, `--recursive` -> `/S` (recursive listing)
/// - `-h`, `--human-readable` -> (sizes are readable by default)
/// - `-S` -> `/O-S` (sort by size descending)
/// - `-t` -> `/O-D` (sort by time descending)
/// - `-r`, `--reverse` -> (reverse sort order)
/// - `-1` -> `/B` (one entry per line, bare format)
#[inline]
fn ls_to_dir_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 30);
    let mut sort_options = Vec::new();

    // -a, --all: show hidden files
    if has_flag(args, 'a', Some("all")) {
        result.push_str(" /A");
    }

    // -R, --recursive: recursive listing
    if has_flag(args, 'R', Some("recursive")) {
        result.push_str(" /S");
    }

    // -1: one entry per line (bare format)
    if has_flag(args, '1', None) {
        result.push_str(" /B");
    }

    // -S: sort by size
    if has_flag(args, 'S', None) {
        sort_options.push(if has_flag(args, 'r', Some("reverse")) {
            "S"
        } else {
            "-S"
        });
    }

    // -t: sort by time
    if has_flag(args, 't', None) {
        sort_options.push(if has_flag(args, 'r', Some("reverse")) {
            "D"
        } else {
            "-D"
        });
    }

    // -X: sort by extension
    if has_flag(args, 'X', None) {
        sort_options.push(if has_flag(args, 'r', Some("reverse")) {
            "E"
        } else {
            "-E"
        });
    }

    // Apply sort options
    if !sort_options.is_empty() {
        result.push_str(" /O");
        for opt in sort_options {
            result.push_str(opt);
        }
    }

    // Extract paths
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

// ============================================================================
// Argument Translators for Windows -> Linux
// ============================================================================

/// Translates Windows dir flags to ls equivalents
/// Supported flags:
/// - `/W` -> wide format (no direct equivalent, use -l for detailed)
/// - `/A` -> `-a` (show hidden files)
/// - `/A:H` -> show only hidden files
/// - `/S` -> `-R` (recursive)
/// - `/B` -> `-1` (bare format, one per line)
/// - `/O:N` -> sort by name
/// - `/O:S` -> `-S` (sort by size)
/// - `/O:D` -> `-t` (sort by date)
/// - `/O:E` -> `-X` (sort by extension)
#[inline]
fn dir_to_ls_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 20);
    let args_upper = args.to_uppercase();

    // /A: show all files including hidden
    if args_upper.contains("/A") {
        result.push_str(" -a");
    }

    // /S: recursive
    if args_upper.contains("/S") {
        result.push_str(" -R");
    }

    // /B: bare format (one per line)
    if args_upper.contains("/B") {
        result.push_str(" -1");
    }

    // Sort options
    if args_upper.contains("/O:S") || args_upper.contains("/O-S") || args_upper.contains("/OS") {
        result.push_str(" -S");
    }
    if args_upper.contains("/O:D") || args_upper.contains("/O-D") || args_upper.contains("/OD") {
        result.push_str(" -t");
    }
    if args_upper.contains("/O:E") || args_upper.contains("/O-E") || args_upper.contains("/OE") {
        result.push_str(" -X");
    }

    // Extract paths (non-flag arguments)
    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates rm flags to Windows del/rd equivalents
/// Supported flags:
/// - `-r`, `-R`, `--recursive` -> `/S` (recursive) - uses rd for directories
/// - `-f`, `--force` -> `/F /Q` (force, quiet)
/// - `-i`, `--interactive` -> `/P` (prompt before delete)
/// - `-v`, `--verbose` -> (no equivalent, ignored)
fn rm_to_del_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 20);

    // Handle recursive flag (handles combined flags like -rf)
    let has_recursive = has_flag(args, 'r', Some("recursive")) || has_flag(args, 'R', None);
    let has_force = has_flag(args, 'f', Some("force"));
    let has_interactive = has_flag(args, 'i', Some("interactive"));

    if has_recursive {
        result.push_str(" /S");
    }
    if has_force {
        result.push_str(" /F /Q");
    }
    if has_interactive {
        result.push_str(" /P");
    }

    // Extract paths
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows del flags to rm equivalents
/// Supported flags:
/// - `/S` -> `-r` (recursive)
/// - `/F` -> `-f` (force)
/// - `/Q` -> `-f` (quiet, treated as force)
/// - `/P` -> `-i` (prompt/interactive)
fn del_to_rm_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 15);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/S") {
        result.push_str(" -r");
    }
    if args_upper.contains("/F") || args_upper.contains("/Q") {
        result.push_str(" -f");
    }
    if args_upper.contains("/P") {
        result.push_str(" -i");
    }

    // Extract paths
    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates cp flags to Windows copy/xcopy equivalents
/// Supported flags:
/// - `-r`, `-R`, `--recursive` -> Uses xcopy /E (recursive with subdirectories)
/// - `-f`, `--force` -> /Y (overwrite without prompt)
/// - `-i`, `--interactive` -> /-Y (prompt before overwrite)
/// - `-v`, `--verbose` -> /F (display full paths)
/// - `-p`, `--preserve` -> /K (preserve attributes)
/// - `-n`, `--no-clobber` -> /-Y (don't overwrite)
fn cp_to_copy_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 20);
    let has_recursive = has_flag(args, 'r', Some("recursive")) || has_flag(args, 'R', None);

    // For recursive copy, we'd use xcopy, but keep it simple for now
    if has_recursive {
        result.push_str(" /E");
    }
    if has_flag(args, 'f', Some("force")) {
        result.push_str(" /Y");
    }
    if has_flag(args, 'i', Some("interactive")) || has_flag(args, 'n', Some("no-clobber")) {
        result.push_str(" /-Y");
    }
    if has_flag(args, 'p', Some("preserve")) {
        result.push_str(" /K");
    }

    // Extract paths (source and destination)
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows copy flags to cp equivalents
/// Supported flags:
/// - `/Y` -> `-f` (force overwrite)
/// - `/-Y` -> `-i` (interactive/prompt)
/// - `/V` -> (verify, no direct equivalent)
fn copy_to_cp_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/Y") && !args_upper.contains("/-Y") {
        result.push_str(" -f");
    }
    if args_upper.contains("/-Y") {
        result.push_str(" -i");
    }

    // Extract paths
    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates mv flags to Windows move equivalents
/// Supported flags:
/// - `-f`, `--force` -> /Y (overwrite without prompt)
/// - `-i`, `--interactive` -> /-Y (prompt before overwrite)
/// - `-n`, `--no-clobber` -> /-Y (don't overwrite)
fn mv_to_move_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);

    if has_flag(args, 'f', Some("force")) {
        result.push_str(" /Y");
    }
    if has_flag(args, 'i', Some("interactive")) || has_flag(args, 'n', Some("no-clobber")) {
        result.push_str(" /-Y");
    }

    // Extract paths
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows move flags to mv equivalents
fn move_to_mv_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/Y") && !args_upper.contains("/-Y") {
        result.push_str(" -f");
    }
    if args_upper.contains("/-Y") {
        result.push_str(" -i");
    }

    // Extract paths
    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates cat flags to Windows type equivalents
/// Supported flags:
/// - `-n`, `--number` -> findstr /N (show line numbers, requires piping)
/// - `-A`, `--show-all` -> (no direct equivalent)
fn cat_to_type_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    // type command just takes filenames, pass through
    // Note: line numbers would require: type file | findstr /N "^"
    let mut result = String::with_capacity(args.len() + 1);

    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows type command to cat
fn type_to_cat_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 1);

    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates grep flags to Windows findstr equivalents
/// Supported flags:
/// - `-i`, `--ignore-case` -> /I (case insensitive)
/// - `-n`, `--line-number` -> /N (show line numbers)
/// - `-r`, `-R`, `--recursive` -> /S (search subdirectories)
/// - `-v`, `--invert-match` -> /V (invert match)
/// - `-c`, `--count` -> /C (count matches, but different syntax)
/// - `-l`, `--files-with-matches` -> /M (print only filenames)
/// - `-w`, `--word-regexp` -> (use /C: for literal, but limited)
/// - `-e PATTERN` -> pattern to search
fn grep_to_findstr_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 20);

    if has_flag(args, 'i', Some("ignore-case")) {
        result.push_str(" /I");
    }
    if has_flag(args, 'n', Some("line-number")) {
        result.push_str(" /N");
    }
    if has_flag(args, 'r', Some("recursive")) || has_flag(args, 'R', None) {
        result.push_str(" /S");
    }
    if has_flag(args, 'v', Some("invert-match")) {
        result.push_str(" /V");
    }
    if has_flag(args, 'l', Some("files-with-matches")) {
        result.push_str(" /M");
    }

    // Extract pattern and files
    // grep pattern file1 file2 -> findstr "pattern" file1 file2
    let parts: Vec<&str> = args.split_whitespace().collect();
    let mut pattern_found = false;

    for (i, part) in parts.iter().enumerate() {
        if part.starts_with('-') {
            // Skip flags and their values
            if *part == "-e" || *part == "--regexp" {
                // Next part is the pattern
                if let Some(pat) = parts.get(i + 1) {
                    result.push_str(" \"");
                    result.push_str(pat);
                    result.push('"');
                    pattern_found = true;
                }
            }
            continue;
        }

        // First non-flag is the pattern (if no -e was used)
        if pattern_found {
            // Rest are files
            result.push(' ');
            result.push_str(part);
        } else {
            result.push_str(" \"");
            result.push_str(part);
            result.push('"');
            pattern_found = true;
        }
    }

    result
}

/// Translates Windows findstr flags to grep equivalents
/// Supported flags:
/// - `/I` -> `-i` (case insensitive)
/// - `/N` -> `-n` (show line numbers)
/// - `/S` -> `-r` (recursive)
/// - `/V` -> `-v` (invert match)
/// - `/M` -> `-l` (print only filenames)
/// - `/C:string` -> literal string search
fn findstr_to_grep_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 15);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/I") {
        result.push_str(" -i");
    }
    if args_upper.contains("/N") {
        result.push_str(" -n");
    }
    if args_upper.contains("/S") {
        result.push_str(" -r");
    }
    if args_upper.contains("/V") {
        result.push_str(" -v");
    }
    if args_upper.contains("/M") {
        result.push_str(" -l");
    }

    // Extract pattern and files
    for part in args.split_whitespace() {
        let part_upper = part.to_uppercase();
        if part_upper.starts_with("/C:") {
            // Literal string search
            result.push_str(" -F \"");
            result.push_str(&part[3..]);
            result.push('"');
        } else if !part.starts_with('/') {
            // Pattern or file
            result.push(' ');
            // Remove quotes if present
            let clean_part = part.trim_matches('"');
            result.push_str(clean_part);
        }
    }

    result
}

/// Translates mkdir flags to Windows md equivalents
/// Supported flags:
/// - `-p`, `--parents` -> (md creates parent dirs by default in Windows)
/// - `-v`, `--verbose` -> (no equivalent)
fn mkdir_to_md_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    // md creates parent directories by default, so -p is not needed
    let mut result = String::with_capacity(args.len());

    for path in extract_paths(args) {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(path);
    }

    result
}

/// Translates Windows md to mkdir
fn md_to_mkdir_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    // Add -p flag since md auto-creates parents
    let mut result = String::with_capacity(args.len() + 5);
    result.push_str("-p");

    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates rmdir flags to Windows rd equivalents
/// Supported flags:
/// - `-r`, `-R` -> /S (recursive)
/// - `-p`, `--parents` -> (remove parent dirs, no direct equivalent)
/// - `--ignore-fail-on-non-empty` -> /Q (quiet)
fn rmdir_to_rd_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);

    if has_flag(args, 'r', Some("recursive")) || has_flag(args, 'R', None) {
        result.push_str(" /S /Q");
    }

    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows rd flags to rmdir equivalents
fn rd_to_rmdir_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/S") {
        result.push_str(" -r");
    }

    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates head flags to Windows PowerShell Get-Content equivalents
/// Supported flags:
/// - `-n NUM`, `--lines=NUM` -> -Head NUM
/// - `-c NUM`, `--bytes=NUM` -> (no direct equivalent)
fn head_to_ps_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return " -Head 10".to_string();
    }

    let mut result = String::with_capacity(args.len() + 20);
    let mut num_lines = "10";

    // Check for -n flag
    if let Some(n) = get_flag_value(args, 'n', Some("lines")) {
        num_lines = n;
    }

    // Extract file path
    for path in extract_paths(args) {
        result.push_str(path);
        result.push(' ');
    }

    result.push_str("-Head ");
    result.push_str(num_lines);

    result
}

/// Translates tail flags to Windows PowerShell Get-Content equivalents
/// Supported flags:
/// - `-n NUM`, `--lines=NUM` -> -Tail NUM
/// - `-f`, `--follow` -> -Wait (follow file)
fn tail_to_ps_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return " -Tail 10".to_string();
    }

    let mut result = String::with_capacity(args.len() + 20);
    let mut num_lines = "10";

    // Check for -n flag
    if let Some(n) = get_flag_value(args, 'n', Some("lines")) {
        num_lines = n;
    }

    // Extract file path
    for path in extract_paths(args) {
        result.push_str(path);
        result.push(' ');
    }

    result.push_str("-Tail ");
    result.push_str(num_lines);

    // -f/--follow -> -Wait
    if has_flag(args, 'f', Some("follow")) {
        result.push_str(" -Wait");
    }

    result
}

/// Translates ping flags between Linux and Windows
/// Linux -> Windows flag mapping:
/// - `-c COUNT` -> `-n COUNT` (number of pings)
/// - `-i INTERVAL` -> (no direct equivalent, Windows default is 1 sec)
/// - `-W TIMEOUT` -> `-w TIMEOUT` (timeout in ms for Windows)
/// - `-s SIZE` -> `-l SIZE` (packet size)
fn ping_linux_to_windows_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);

    // -c COUNT -> -n COUNT
    if let Some(count) = get_flag_value(args, 'c', None) {
        result.push_str(" -n ");
        result.push_str(count);
    }

    // -W TIMEOUT -> -w TIMEOUT (Linux uses seconds, Windows uses ms)
    if let Some(timeout) = get_flag_value(args, 'W', None) {
        if let Ok(t) = timeout.parse::<u32>() {
            result.push_str(" -w ");
            result.push_str(&(t * 1000).to_string());
        }
    }

    // -s SIZE -> -l SIZE
    if let Some(size) = get_flag_value(args, 's', None) {
        result.push_str(" -l ");
        result.push_str(size);
    }

    // Extract host
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates ping flags from Windows to Linux
fn ping_windows_to_linux_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let parts: Vec<&str> = args.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        let part_lower = part.to_lowercase();

        // -n COUNT -> -c COUNT
        if part_lower == "-n" {
            if let Some(count) = parts.get(i + 1) {
                result.push_str(" -c ");
                result.push_str(count);
            }
        }
        // -w TIMEOUT -> -W TIMEOUT (Windows ms to Linux seconds)
        else if part_lower == "-w" {
            if let Some(timeout) = parts.get(i + 1) {
                if let Ok(t) = timeout.parse::<u32>() {
                    result.push_str(" -W ");
                    result.push_str(&(t / 1000).max(1).to_string());
                }
            }
        }
        // -l SIZE -> -s SIZE
        else if part_lower == "-l" {
            if let Some(size) = parts.get(i + 1) {
                result.push_str(" -s ");
                result.push_str(size);
            }
        }
        // Host or other argument
        else if !part.starts_with('-') && !part.starts_with('/') {
            // Skip values that follow flags
            if i > 0 {
                let prev = parts[i - 1].to_lowercase();
                if prev == "-n" || prev == "-w" || prev == "-l" || prev == "-t" {
                    continue;
                }
            }
            result.push(' ');
            result.push_str(part);
        }
    }

    result
}

/// Translates kill arguments to Windows taskkill equivalents
/// Supported flags:
/// - `-9`, `--signal=KILL` -> /F (force kill)
/// - `-15`, `--signal=TERM` -> (normal termination)
/// - PID -> /PID PID
fn kill_to_taskkill_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 15);
    let has_force = has_flag(args, '9', None) || args.contains("KILL") || args.contains("SIGKILL");

    if has_force {
        result.push_str(" /F");
    }

    // Extract PIDs
    for part in args.split_whitespace() {
        if !part.starts_with('-') {
            // It's a PID
            result.push_str(" /PID ");
            result.push_str(part);
        }
    }

    result
}

/// Translates Windows taskkill flags to kill equivalents
fn taskkill_to_kill_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let args_upper = args.to_uppercase();

    // /F -> -9
    if args_upper.contains("/F") {
        result.push_str(" -9");
    }

    // Extract PID from /PID value
    let parts: Vec<&str> = args.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.to_uppercase() == "/PID" {
            if let Some(pid) = parts.get(i + 1) {
                result.push(' ');
                result.push_str(pid);
            }
        }
        // /IM name -> use pkill (but for kill, we just note it)
        else if part.to_uppercase() == "/IM" {
            // Can't directly translate process name to kill
            // Would need pkill or killall
        }
    }

    result
}

/// Translates echo flags - mostly pass-through but handle -n (no newline)
fn echo_linux_to_windows_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    // Windows echo doesn't support -n, but we can use set /p for no newline
    // For simplicity, just extract the message
    let mut result = String::with_capacity(args.len());

    for part in args.split_whitespace() {
        if !part.starts_with('-') {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(part);
        }
    }

    result
}

/// Translates sort flags between Linux and Windows
/// Linux -> Windows:
/// - `-r`, `--reverse` -> /R (reverse)
/// - `-n`, `--numeric-sort` -> (Windows sort doesn't have this)
/// - `-u`, `--unique` -> (pipe to uniq/Get-Unique)
fn sort_linux_to_windows_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);

    if has_flag(args, 'r', Some("reverse")) {
        result.push_str(" /R");
    }

    // Extract file paths
    for path in extract_paths(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

/// Translates Windows sort flags to Linux sort equivalents
fn sort_windows_to_linux_args(args: &str) -> String {
    let args = args.trim();

    if args.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(args.len() + 10);
    let args_upper = args.to_uppercase();

    if args_upper.contains("/R") {
        result.push_str(" -r");
    }

    // Extract file paths
    for path in extract_paths_windows(args) {
        result.push(' ');
        result.push_str(path);
    }

    result
}

// ============================================================================
// Static Command Mappings
// ============================================================================

static LINUX_TO_WINDOWS_MAP: LazyLock<HashMap<&'static str, CommandMapping>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // ========== File System Commands ==========

        m.insert(
            "ls",
            CommandMapping {
                target_cmd: "dir",
                description: "List directory contents",
                arg_translator: ls_to_dir_args,
            },
        );

        m.insert(
            "pwd",
            CommandMapping {
                target_cmd: "cd",
                description: "Print working directory",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "cat",
            CommandMapping {
                target_cmd: "type",
                description: "Display file contents",
                arg_translator: cat_to_type_args,
            },
        );

        m.insert(
            "rm",
            CommandMapping {
                target_cmd: "del",
                description: "Remove files",
                arg_translator: rm_to_del_args,
            },
        );

        m.insert(
            "cp",
            CommandMapping {
                target_cmd: "copy",
                description: "Copy files",
                arg_translator: cp_to_copy_args,
            },
        );

        m.insert(
            "mv",
            CommandMapping {
                target_cmd: "move",
                description: "Move/rename files",
                arg_translator: mv_to_move_args,
            },
        );

        m.insert(
            "mkdir",
            CommandMapping {
                target_cmd: "md",
                description: "Create directory",
                arg_translator: mkdir_to_md_args,
            },
        );

        m.insert(
            "rmdir",
            CommandMapping {
                target_cmd: "rd",
                description: "Remove directory",
                arg_translator: rmdir_to_rd_args,
            },
        );

        m.insert(
            "touch",
            CommandMapping {
                target_cmd: "type nul >",
                description: "Create empty file",
                arg_translator: |args| args.trim().to_string(),
            },
        );

        m.insert(
            "ln",
            CommandMapping {
                target_cmd: "mklink",
                description: "Create symbolic link",
                arg_translator: |args| {
                    // ln -s target link -> mklink link target (reversed order)
                    let parts: Vec<&str> = args
                        .split_whitespace()
                        .filter(|p| !p.starts_with('-'))
                        .collect();
                    if parts.len() >= 2 {
                        format!(" {} {}", parts[1], parts[0])
                    } else {
                        args.to_string()
                    }
                },
            },
        );

        // ========== Text Processing Commands ==========

        m.insert(
            "grep",
            CommandMapping {
                target_cmd: "findstr",
                description: "Search text patterns",
                arg_translator: grep_to_findstr_args,
            },
        );

        m.insert(
            "head",
            CommandMapping {
                target_cmd: "powershell Get-Content",
                description: "Display first lines of file",
                arg_translator: head_to_ps_args,
            },
        );

        m.insert(
            "tail",
            CommandMapping {
                target_cmd: "powershell Get-Content",
                description: "Display last lines of file",
                arg_translator: tail_to_ps_args,
            },
        );

        m.insert(
            "wc",
            CommandMapping {
                target_cmd: "find /c /v \"\"",
                description: "Count lines/words/characters",
                arg_translator: |args| {
                    // wc -l file -> find /c /v "" file
                    let mut result = String::new();
                    for path in extract_paths(args) {
                        result.push(' ');
                        result.push_str(path);
                    }
                    result
                },
            },
        );

        m.insert(
            "sort",
            CommandMapping {
                target_cmd: "sort",
                description: "Sort lines",
                arg_translator: sort_linux_to_windows_args,
            },
        );

        m.insert(
            "uniq",
            CommandMapping {
                target_cmd: "powershell Get-Unique",
                description: "Filter duplicate lines",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "cut",
            CommandMapping {
                target_cmd: "powershell ForEach-Object",
                description: "Extract columns (limited translation)",
                arg_translator: |args| {
                    // Basic cut -d',' -f1 translation - extracts first comma-delimited field
                    // Complex cut commands may need manual adjustment
                    let paths = extract_paths(args);
                    if paths.is_empty() {
                        " { $_.Split(',')[0] }".to_string()
                    } else {
                        format!(" {{ $_.Split(',')[0] }} {}", paths.join(" "))
                    }
                },
            },
        );

        m.insert(
            "tr",
            CommandMapping {
                target_cmd: "powershell -replace",
                description: "Translate characters",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "diff",
            CommandMapping {
                target_cmd: "fc",
                description: "Compare files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "tee",
            CommandMapping {
                target_cmd: "powershell Tee-Object",
                description: "Read from stdin and write to stdout and files",
                arg_translator: identity_args,
            },
        );

        // ========== Viewing Commands ==========

        m.insert(
            "less",
            CommandMapping {
                target_cmd: "more",
                description: "Page through file",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "more",
            CommandMapping {
                target_cmd: "more",
                description: "Page through file",
                arg_translator: identity_args,
            },
        );

        // ========== System Commands ==========

        m.insert(
            "clear",
            CommandMapping {
                target_cmd: "cls",
                description: "Clear screen",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "which",
            CommandMapping {
                target_cmd: "where",
                description: "Locate command",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "whoami",
            CommandMapping {
                target_cmd: "whoami",
                description: "Display current user",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "hostname",
            CommandMapping {
                target_cmd: "hostname",
                description: "Display system hostname",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "env",
            CommandMapping {
                target_cmd: "set",
                description: "Display environment variables",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "printenv",
            CommandMapping {
                target_cmd: "set",
                description: "Display environment variables",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "export",
            CommandMapping {
                target_cmd: "set",
                description: "Set environment variable",
                arg_translator: |args| {
                    // export VAR=value -> set VAR=value
                    // Just pass through the args
                    format!(" {}", args.trim())
                },
            },
        );

        m.insert(
            "uname",
            CommandMapping {
                target_cmd: "ver",
                description: "Display system information",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "uptime",
            CommandMapping {
                target_cmd: "net statistics workstation",
                description: "Display system uptime (statistics since boot)",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "date",
            CommandMapping {
                target_cmd: "date /t",
                description: "Display current date",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "echo",
            CommandMapping {
                target_cmd: "echo",
                description: "Print text",
                arg_translator: echo_linux_to_windows_args,
            },
        );

        m.insert(
            "alias",
            CommandMapping {
                target_cmd: "doskey",
                description: "Create command alias",
                arg_translator: |args| {
                    // alias name='command' -> doskey name=command
                    args.replace('\'', "").replace("=\"", "=")
                },
            },
        );

        m.insert(
            "history",
            CommandMapping {
                target_cmd: "doskey /history",
                description: "Show command history",
                arg_translator: |_| String::new(),
            },
        );

        // ========== Process Commands ==========

        m.insert(
            "ps",
            CommandMapping {
                target_cmd: "tasklist",
                description: "List processes",
                arg_translator: |args| {
                    // ps aux -> tasklist /V
                    if args.contains("aux") || args.contains("-e") || args.contains("-A") {
                        " /V".to_string()
                    } else {
                        String::new()
                    }
                },
            },
        );

        m.insert(
            "kill",
            CommandMapping {
                target_cmd: "taskkill",
                description: "Terminate process",
                arg_translator: kill_to_taskkill_args,
            },
        );

        m.insert(
            "killall",
            CommandMapping {
                target_cmd: "taskkill /IM",
                description: "Kill processes by name",
                arg_translator: |args| {
                    // killall firefox -> taskkill /IM firefox.exe /F
                    let mut result = String::new();
                    for name in args.split_whitespace().filter(|p| !p.starts_with('-')) {
                        if !result.is_empty() {
                            result.push_str(" & taskkill /IM");
                        }
                        result.push(' ');
                        result.push_str(name);
                        if !name.to_lowercase().ends_with(".exe") {
                            result.push_str(".exe");
                        }
                    }
                    if args.contains("-9") || args.contains("-KILL") {
                        result.push_str(" /F");
                    }
                    result
                },
            },
        );

        m.insert(
            "top",
            CommandMapping {
                target_cmd: "tasklist /V",
                description: "Display processes",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "pgrep",
            CommandMapping {
                target_cmd: "tasklist /FI",
                description: "Search processes",
                arg_translator: |args| format!(" \"IMAGENAME eq {}*\"", args.trim()),
            },
        );

        // ========== Disk Commands ==========

        m.insert(
            "df",
            CommandMapping {
                target_cmd: "powershell Get-PSDrive -PSProvider FileSystem",
                description: "Display disk space",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "du",
            CommandMapping {
                target_cmd: "dir /S",
                description: "Display disk usage",
                arg_translator: |args| {
                    let mut result = String::new();
                    for path in extract_paths(args) {
                        result.push(' ');
                        result.push_str(path);
                    }
                    result
                },
            },
        );

        m.insert(
            "mount",
            CommandMapping {
                target_cmd: "mountvol",
                description: "Mount filesystem",
                arg_translator: identity_args,
            },
        );

        // ========== Network Commands ==========

        m.insert(
            "ping",
            CommandMapping {
                target_cmd: "ping",
                description: "Ping host",
                arg_translator: ping_linux_to_windows_args,
            },
        );

        m.insert(
            "curl",
            CommandMapping {
                target_cmd: "powershell Invoke-WebRequest",
                description: "Transfer data from URL",
                arg_translator: |args| {
                    // curl -o file url -> Invoke-WebRequest -Uri url -OutFile file
                    let mut result = String::new();
                    let parts: Vec<&str> = args.split_whitespace().collect();

                    let mut output_file = None;
                    let mut url = None;
                    let mut i = 0;

                    while i < parts.len() {
                        let part = parts[i];
                        if part == "-o" || part == "--output" {
                            if let Some(file) = parts.get(i + 1) {
                                output_file = Some(*file);
                                i += 1;
                            }
                        } else if !part.starts_with('-') {
                            url = Some(part);
                        }
                        i += 1;
                    }

                    if let Some(u) = url {
                        result.push_str(" -Uri ");
                        result.push_str(u);
                    }
                    if let Some(f) = output_file {
                        result.push_str(" -OutFile ");
                        result.push_str(f);
                    }

                    result
                },
            },
        );

        m.insert(
            "wget",
            CommandMapping {
                target_cmd: "powershell Invoke-WebRequest -OutFile",
                description: "Download file from URL",
                arg_translator: |args| {
                    // wget url -O file -> Invoke-WebRequest -Uri url -OutFile file
                    let mut result = String::new();
                    let parts: Vec<&str> = args.split_whitespace().collect();

                    let mut output_file = None;
                    let mut url = None;

                    let mut i = 0;
                    while i < parts.len() {
                        let part = parts[i];
                        if part == "-O" || part == "--output-document" {
                            if let Some(file) = parts.get(i + 1) {
                                output_file = Some(*file);
                                i += 1;
                            }
                        } else if !part.starts_with('-') {
                            url = Some(part);
                        }
                        i += 1;
                    }

                    if let Some(f) = output_file {
                        result.push_str(f);
                    } else if let Some(u) = url {
                        // Default output filename from URL
                        if let Some(filename) = u.rsplit('/').next() {
                            result.push_str(filename);
                        }
                    }

                    if let Some(u) = url {
                        result.push_str(" -Uri ");
                        result.push_str(u);
                    }

                    result
                },
            },
        );

        m.insert(
            "ifconfig",
            CommandMapping {
                target_cmd: "ipconfig",
                description: "Network interface configuration",
                arg_translator: |args| {
                    if args.contains("-a") || args.trim().is_empty() {
                        " /all".to_string()
                    } else {
                        String::new()
                    }
                },
            },
        );

        m.insert(
            "ip",
            CommandMapping {
                target_cmd: "ipconfig",
                description: "Network configuration",
                arg_translator: |args| {
                    if args.contains("addr") || args.contains("address") {
                        " /all".to_string()
                    } else {
                        String::new()
                    }
                },
            },
        );

        m.insert(
            "netstat",
            CommandMapping {
                target_cmd: "netstat",
                description: "Network statistics",
                arg_translator: |args| {
                    let mut result = String::new();
                    if has_flag(args, 'a', Some("all")) {
                        result.push_str(" -a");
                    }
                    if has_flag(args, 'n', Some("numeric")) {
                        result.push_str(" -n");
                    }
                    if has_flag(args, 'p', Some("program")) {
                        result.push_str(" -b");
                    }
                    if has_flag(args, 't', Some("tcp")) {
                        result.push_str(" -p TCP");
                    }
                    if has_flag(args, 'u', Some("udp")) {
                        result.push_str(" -p UDP");
                    }
                    result
                },
            },
        );

        m.insert(
            "traceroute",
            CommandMapping {
                target_cmd: "tracert",
                description: "Trace route to host",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "nslookup",
            CommandMapping {
                target_cmd: "nslookup",
                description: "DNS lookup",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "ssh",
            CommandMapping {
                target_cmd: "ssh",
                description: "Secure shell",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "scp",
            CommandMapping {
                target_cmd: "scp",
                description: "Secure copy",
                arg_translator: identity_args,
            },
        );

        // ========== Archive Commands ==========

        m.insert(
            "tar",
            CommandMapping {
                target_cmd: "tar",
                description: "Archive files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "zip",
            CommandMapping {
                target_cmd: "powershell Compress-Archive",
                description: "Create zip archive",
                arg_translator: |args| {
                    // zip archive.zip file1 file2 -> Compress-Archive -Path file1,file2 -DestinationPath archive.zip
                    let parts: Vec<&str> = args
                        .split_whitespace()
                        .filter(|p| !p.starts_with('-'))
                        .collect();
                    if parts.len() >= 2 {
                        let archive = parts[0];
                        let files: Vec<&str> = parts[1..].to_vec();
                        format!(" -Path {} -DestinationPath {}", files.join(","), archive)
                    } else {
                        args.to_string()
                    }
                },
            },
        );

        m.insert(
            "unzip",
            CommandMapping {
                target_cmd: "powershell Expand-Archive",
                description: "Extract zip archive",
                arg_translator: |args| {
                    // unzip archive.zip -d dir -> Expand-Archive -Path archive.zip -DestinationPath dir
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    let mut archive = "";
                    let mut dest_dir = ".";

                    let mut i = 0;
                    while i < parts.len() {
                        let part = parts[i];
                        if part == "-d" {
                            if let Some(dir) = parts.get(i + 1) {
                                dest_dir = dir;
                                i += 1;
                            }
                        } else if !part.starts_with('-') {
                            archive = part;
                        }
                        i += 1;
                    }

                    format!(" -Path {} -DestinationPath {}", archive, dest_dir)
                },
            },
        );

        m.insert(
            "gzip",
            CommandMapping {
                target_cmd: "powershell Compress-Archive",
                description: "Gzip compression",
                arg_translator: |args| {
                    let file = args
                        .split_whitespace()
                        .find(|p| !p.starts_with('-'))
                        .unwrap_or("");
                    format!(" -Path {} -DestinationPath {}.gz", file, file)
                },
            },
        );

        m.insert(
            "gunzip",
            CommandMapping {
                target_cmd: "powershell Expand-Archive",
                description: "Gzip decompression",
                arg_translator: identity_args,
            },
        );

        // ========== Permission Commands ==========

        m.insert(
            "chmod",
            CommandMapping {
                target_cmd: "icacls",
                description: "Change file permissions",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "chown",
            CommandMapping {
                target_cmd: "icacls",
                description: "Change file owner",
                arg_translator: identity_args,
            },
        );

        // ========== Find Commands ==========

        m.insert(
            "find",
            CommandMapping {
                target_cmd: "dir /S /B",
                description: "Find files",
                arg_translator: |args| {
                    // find /path -name "pattern" -> dir /S /B path\*pattern*
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    let mut path = ".";
                    let mut pattern = "*";

                    let mut i = 0;
                    while i < parts.len() {
                        let part = parts[i];
                        if part == "-name" || part == "-iname" {
                            if let Some(p) = parts.get(i + 1) {
                                pattern = p.trim_matches('"').trim_matches('\'');
                                i += 1;
                            }
                        } else if !part.starts_with('-') {
                            path = part;
                        }
                        i += 1;
                    }

                    format!(" {}\\{}", path, pattern)
                },
            },
        );

        m.insert(
            "locate",
            CommandMapping {
                target_cmd: "dir /S /B C:\\",
                description: "Find files by name",
                arg_translator: |args| format!("*{}*", args.trim()),
            },
        );

        // ========== Text Output Commands ==========

        m.insert(
            "printf",
            CommandMapping {
                target_cmd: "echo",
                description: "Format and print",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "file",
            CommandMapping {
                target_cmd: "powershell Get-Item",
                description: "Determine file type",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "stat",
            CommandMapping {
                target_cmd: "powershell Get-Item",
                description: "Display file status",
                arg_translator: |args| format!("{} | Format-List *", args.trim()),
            },
        );

        m.insert(
            "basename",
            CommandMapping {
                target_cmd: "powershell Split-Path -Leaf",
                description: "Strip directory from filename",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "dirname",
            CommandMapping {
                target_cmd: "powershell Split-Path -Parent",
                description: "Strip filename from path",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "realpath",
            CommandMapping {
                target_cmd: "powershell Resolve-Path",
                description: "Print resolved path",
                arg_translator: identity_args,
            },
        );

        // ========== Exit/Logout Commands ==========

        m.insert(
            "exit",
            CommandMapping {
                target_cmd: "exit",
                description: "Exit shell",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "logout",
            CommandMapping {
                target_cmd: "exit",
                description: "Logout from shell",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "shutdown",
            CommandMapping {
                target_cmd: "shutdown",
                description: "Shutdown system",
                arg_translator: |args| {
                    let mut result = String::new();
                    if has_flag(args, 'r', Some("reboot")) {
                        result.push_str(" /r");
                    } else {
                        result.push_str(" /s");
                    }
                    if has_flag(args, 'h', Some("halt")) {
                        result.push_str(" /s");
                    }
                    if has_flag(args, 'c', Some("cancel")) {
                        result.push_str(" /a");
                    }
                    result.push_str(" /t 0");
                    result
                },
            },
        );

        m.insert(
            "reboot",
            CommandMapping {
                target_cmd: "shutdown /r /t 0",
                description: "Reboot system",
                arg_translator: |_| String::new(),
            },
        );

        m
    });

static WINDOWS_TO_LINUX_MAP: LazyLock<HashMap<&'static str, CommandMapping>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // ========== File System Commands ==========

        m.insert(
            "dir",
            CommandMapping {
                target_cmd: "ls",
                description: "List directory contents",
                arg_translator: dir_to_ls_args,
            },
        );

        m.insert(
            "cd",
            CommandMapping {
                target_cmd: "pwd",
                description: "Print working directory (when used alone)",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "type",
            CommandMapping {
                target_cmd: "cat",
                description: "Display file contents",
                arg_translator: type_to_cat_args,
            },
        );

        m.insert(
            "del",
            CommandMapping {
                target_cmd: "rm",
                description: "Remove files",
                arg_translator: del_to_rm_args,
            },
        );

        m.insert(
            "erase",
            CommandMapping {
                target_cmd: "rm",
                description: "Remove files",
                arg_translator: del_to_rm_args,
            },
        );

        m.insert(
            "copy",
            CommandMapping {
                target_cmd: "cp",
                description: "Copy files",
                arg_translator: copy_to_cp_args,
            },
        );

        m.insert(
            "xcopy",
            CommandMapping {
                target_cmd: "cp -r",
                description: "Copy files and directories",
                arg_translator: |args| {
                    let mut result = String::new();
                    for path in extract_paths_windows(args) {
                        result.push(' ');
                        result.push_str(path);
                    }
                    result
                },
            },
        );

        m.insert(
            "robocopy",
            CommandMapping {
                target_cmd: "rsync",
                description: "Robust copy",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "move",
            CommandMapping {
                target_cmd: "mv",
                description: "Move/rename files",
                arg_translator: move_to_mv_args,
            },
        );

        m.insert(
            "ren",
            CommandMapping {
                target_cmd: "mv",
                description: "Rename files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "rename",
            CommandMapping {
                target_cmd: "mv",
                description: "Rename files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "md",
            CommandMapping {
                target_cmd: "mkdir",
                description: "Create directory",
                arg_translator: md_to_mkdir_args,
            },
        );

        m.insert(
            "mkdir",
            CommandMapping {
                target_cmd: "mkdir",
                description: "Create directory",
                arg_translator: md_to_mkdir_args,
            },
        );

        m.insert(
            "rd",
            CommandMapping {
                target_cmd: "rmdir",
                description: "Remove directory",
                arg_translator: rd_to_rmdir_args,
            },
        );

        m.insert(
            "rmdir",
            CommandMapping {
                target_cmd: "rmdir",
                description: "Remove directory",
                arg_translator: rd_to_rmdir_args,
            },
        );

        m.insert(
            "mklink",
            CommandMapping {
                target_cmd: "ln -s",
                description: "Create symbolic link",
                arg_translator: |args| {
                    // mklink link target -> ln -s target link (reversed order)
                    let parts: Vec<&str> = args
                        .split_whitespace()
                        .filter(|p| !p.starts_with('/'))
                        .collect();
                    if parts.len() >= 2 {
                        format!(" {} {}", parts[1], parts[0])
                    } else {
                        args.to_string()
                    }
                },
            },
        );

        m.insert(
            "attrib",
            CommandMapping {
                target_cmd: "chmod",
                description: "Change file attributes",
                arg_translator: identity_args,
            },
        );

        // ========== Text Processing Commands ==========

        m.insert(
            "findstr",
            CommandMapping {
                target_cmd: "grep",
                description: "Search text patterns",
                arg_translator: findstr_to_grep_args,
            },
        );

        m.insert(
            "find",
            CommandMapping {
                target_cmd: "grep",
                description: "Search text in files",
                arg_translator: |args| {
                    // find "string" file -> grep "string" file
                    let args_upper = args.to_uppercase();
                    let mut result = String::new();

                    if args_upper.contains("/I") {
                        result.push_str(" -i");
                    }
                    if args_upper.contains("/N") {
                        result.push_str(" -n");
                    }
                    if args_upper.contains("/C") {
                        result.push_str(" -c");
                    }
                    if args_upper.contains("/V") {
                        result.push_str(" -v");
                    }

                    // Extract search string and file
                    for part in args.split_whitespace() {
                        if !part.starts_with('/') {
                            result.push(' ');
                            result.push_str(part);
                        }
                    }

                    result
                },
            },
        );

        m.insert(
            "sort",
            CommandMapping {
                target_cmd: "sort",
                description: "Sort lines",
                arg_translator: sort_windows_to_linux_args,
            },
        );

        m.insert(
            "fc",
            CommandMapping {
                target_cmd: "diff",
                description: "Compare files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "comp",
            CommandMapping {
                target_cmd: "diff",
                description: "Compare files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "more",
            CommandMapping {
                target_cmd: "less",
                description: "Page through file",
                arg_translator: identity_args,
            },
        );

        // ========== System Commands ==========

        m.insert(
            "cls",
            CommandMapping {
                target_cmd: "clear",
                description: "Clear screen",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "where",
            CommandMapping {
                target_cmd: "which",
                description: "Locate command",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "whoami",
            CommandMapping {
                target_cmd: "whoami",
                description: "Display current user",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "hostname",
            CommandMapping {
                target_cmd: "hostname",
                description: "Display system hostname",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "set",
            CommandMapping {
                target_cmd: "printenv",
                description: "Display environment variables",
                arg_translator: |args| {
                    if args.trim().is_empty() {
                        String::new()
                    } else {
                        // For setting, this maps to env display only
                        // Setting env vars has no direct equivalent
                        format!(" {}", args.split('=').next().unwrap_or(""))
                    }
                },
            },
        );

        m.insert(
            "ver",
            CommandMapping {
                target_cmd: "uname -a",
                description: "Display system version",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "systeminfo",
            CommandMapping {
                target_cmd: "uname -a && cat /etc/os-release",
                description: "Display system information",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "echo",
            CommandMapping {
                target_cmd: "echo",
                description: "Print text",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "doskey",
            CommandMapping {
                target_cmd: "alias",
                description: "Create command alias",
                arg_translator: |args| {
                    // doskey name=command -> alias name='command'
                    if let Some((name, cmd)) = args.split_once('=') {
                        format!(" {}='{}'", name.trim(), cmd.trim())
                    } else {
                        args.to_string()
                    }
                },
            },
        );

        m.insert(
            "title",
            CommandMapping {
                target_cmd: "echo -ne \"\\033]0;",
                description: "Set terminal title",
                arg_translator: |args| format!("{}\\007\"", args.trim()),
            },
        );

        m.insert(
            "color",
            CommandMapping {
                target_cmd: "echo -e",
                description: "Set console colors (via ANSI codes)",
                arg_translator: |_| " '\\033[0m'".to_string(),
            },
        );

        // ========== Process Commands ==========

        m.insert(
            "tasklist",
            CommandMapping {
                target_cmd: "ps",
                description: "List processes",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    let mut result = String::new();

                    if args_upper.contains("/V") {
                        result.push_str(" aux");
                    } else {
                        result.push_str(" -e");
                    }

                    // Filter by image name - tasklist /FI maps to ps aux | grep
                    // We just return ps aux here, user can pipe to grep manually
                    if args_upper.contains("/FI") {
                        // Extract filter pattern if possible
                        if let Some(pos) = args.to_uppercase().find("IMAGENAME EQ") {
                            let rest = &args[pos + 12..];
                            if let Some(name) = rest.split_whitespace().next() {
                                let name = name.trim_matches('"').trim_matches('*');
                                result.push_str(" aux | grep ");
                                result.push_str(name);
                            }
                        }
                    }

                    result
                },
            },
        );

        m.insert(
            "taskkill",
            CommandMapping {
                target_cmd: "kill",
                description: "Terminate process",
                arg_translator: taskkill_to_kill_args,
            },
        );

        m.insert(
            "start",
            CommandMapping {
                target_cmd: "xdg-open",
                description: "Start/open file or program",
                arg_translator: |args| {
                    // Filter out Windows-specific flags
                    let mut result = String::new();
                    for part in args.split_whitespace() {
                        if !part.starts_with('/') {
                            if !result.is_empty() {
                                result.push(' ');
                            }
                            result.push_str(part);
                        }
                    }
                    result
                },
            },
        );

        // ========== Disk Commands ==========

        m.insert(
            "chkdsk",
            CommandMapping {
                target_cmd: "fsck",
                description: "Check disk",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "diskpart",
            CommandMapping {
                target_cmd: "fdisk -l",
                description: "Disk partitioning (list mode)",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "format",
            CommandMapping {
                target_cmd: "mkfs",
                description: "Format disk",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "mountvol",
            CommandMapping {
                target_cmd: "mount",
                description: "Mount volume",
                arg_translator: identity_args,
            },
        );

        // ========== Network Commands ==========

        m.insert(
            "ping",
            CommandMapping {
                target_cmd: "ping",
                description: "Ping host",
                arg_translator: ping_windows_to_linux_args,
            },
        );

        m.insert(
            "ipconfig",
            CommandMapping {
                target_cmd: "ip addr",
                description: "Network configuration",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    if args_upper.contains("/ALL") {
                        " show".to_string()
                    } else if args_upper.contains("/RELEASE") {
                        // dhclient -r releases DHCP lease
                        String::new() // Return empty, requires different command
                    } else if args_upper.contains("/RENEW") {
                        // dhclient renews DHCP lease
                        String::new() // Return empty, requires different command
                    } else if args_upper.contains("/FLUSHDNS") {
                        // DNS flush is handled differently
                        String::new() // Return empty, requires different command
                    } else {
                        " show".to_string()
                    }
                },
            },
        );

        m.insert(
            "netstat",
            CommandMapping {
                target_cmd: "netstat",
                description: "Network statistics",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    let mut result = String::new();

                    if args_upper.contains("-A") {
                        result.push_str(" -a");
                    }
                    if args_upper.contains("-N") {
                        result.push_str(" -n");
                    }
                    if args_upper.contains("-B") {
                        result.push_str(" -p");
                    }
                    if args_upper.contains("-P TCP") {
                        result.push_str(" -t");
                    }
                    if args_upper.contains("-P UDP") {
                        result.push_str(" -u");
                    }

                    result
                },
            },
        );

        m.insert(
            "tracert",
            CommandMapping {
                target_cmd: "traceroute",
                description: "Trace route to host",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "nslookup",
            CommandMapping {
                target_cmd: "nslookup",
                description: "DNS lookup",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "netsh",
            CommandMapping {
                target_cmd: "ip",
                description: "Network shell",
                arg_translator: |args| {
                    // netsh has many subcommands - provide basic translation
                    let args_lower = args.to_lowercase();
                    if args_lower.contains("interface") && args_lower.contains("show") {
                        " addr show".to_string()
                    } else if args_lower.contains("firewall") {
                        // Firewall commands map to iptables but need different syntax
                        String::new()
                    } else {
                        String::new()
                    }
                },
            },
        );

        m.insert(
            "route",
            CommandMapping {
                target_cmd: "ip route",
                description: "Display/modify routing table",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    if args_upper.contains("PRINT") {
                        " show".to_string()
                    } else if args_upper.contains("ADD") {
                        " add".to_string()
                    } else if args_upper.contains("DELETE") {
                        " del".to_string()
                    } else {
                        " show".to_string()
                    }
                },
            },
        );

        m.insert(
            "arp",
            CommandMapping {
                target_cmd: "arp",
                description: "Display/modify ARP cache",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    if args_upper.contains("-A") {
                        " -a".to_string()
                    } else {
                        args.to_string()
                    }
                },
            },
        );

        // ========== Archive Commands ==========

        m.insert(
            "tar",
            CommandMapping {
                target_cmd: "tar",
                description: "Archive files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "expand",
            CommandMapping {
                target_cmd: "unzip",
                description: "Expand compressed files",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "compact",
            CommandMapping {
                target_cmd: "gzip",
                description: "Compress files",
                arg_translator: identity_args,
            },
        );

        // ========== Help/Info Commands ==========

        m.insert(
            "help",
            CommandMapping {
                target_cmd: "man",
                description: "Display help",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "/?",
            CommandMapping {
                target_cmd: "--help",
                description: "Display help",
                arg_translator: |_| String::new(),
            },
        );

        // ========== Exit/Shutdown Commands ==========

        m.insert(
            "exit",
            CommandMapping {
                target_cmd: "exit",
                description: "Exit shell",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "logoff",
            CommandMapping {
                target_cmd: "logout",
                description: "Log off",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "shutdown",
            CommandMapping {
                target_cmd: "shutdown",
                description: "Shutdown system",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    let mut result = String::new();

                    if args_upper.contains("/R") {
                        result.push_str(" -r");
                    }
                    if args_upper.contains("/S") {
                        result.push_str(" -h");
                    }
                    if args_upper.contains("/A") {
                        result.push_str(" -c");
                    }
                    if args_upper.contains("/T") {
                        // Extract time value
                        let parts: Vec<&str> = args.split_whitespace().collect();
                        for (i, part) in parts.iter().enumerate() {
                            if part.to_uppercase() == "/T" {
                                if let Some(time) = parts.get(i + 1) {
                                    result.push_str(" +");
                                    // Convert seconds to minutes
                                    if let Ok(secs) = time.parse::<u32>() {
                                        result.push_str(&(secs / 60).to_string());
                                    } else {
                                        result.push_str(time);
                                    }
                                }
                            }
                        }
                    } else {
                        result.push_str(" now");
                    }

                    result
                },
            },
        );

        // ========== Date/Time Commands ==========

        m.insert(
            "date",
            CommandMapping {
                target_cmd: "date",
                description: "Display date",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    if args_upper.contains("/T") {
                        String::new()
                    } else {
                        // Interactive date setting has no direct equivalent
                        String::new()
                    }
                },
            },
        );

        m.insert(
            "time",
            CommandMapping {
                target_cmd: "date +%T",
                description: "Display time",
                arg_translator: |args| {
                    let args_upper = args.to_uppercase();
                    if args_upper.contains("/T") {
                        String::new()
                    } else {
                        // Interactive time setting has no direct equivalent
                        String::new()
                    }
                },
            },
        );

        // ========== Path Commands ==========

        m.insert(
            "path",
            CommandMapping {
                target_cmd: "echo $PATH",
                description: "Display path",
                arg_translator: |_| String::new(),
            },
        );

        m.insert(
            "pushd",
            CommandMapping {
                target_cmd: "pushd",
                description: "Push directory onto stack",
                arg_translator: identity_args,
            },
        );

        m.insert(
            "popd",
            CommandMapping {
                target_cmd: "popd",
                description: "Pop directory from stack",
                arg_translator: identity_args,
            },
        );

        // ========== Registry/Service Commands ==========

        m.insert(
            "reg",
            CommandMapping {
                target_cmd: "echo",
                description: "Registry operations (no Linux equivalent)",
                arg_translator: |_| " 'Registry is Windows-only'".to_string(),
            },
        );

        m.insert(
            "sc",
            CommandMapping {
                target_cmd: "systemctl",
                description: "Service control",
                arg_translator: |args| {
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    if parts.is_empty() {
                        return String::new();
                    }

                    match parts[0].to_lowercase().as_str() {
                        "query" => " status".to_string(),
                        "start" => format!(" start {}", parts.get(1).unwrap_or(&"")),
                        "stop" => format!(" stop {}", parts.get(1).unwrap_or(&"")),
                        "config" => {
                            // systemctl edit allows editing service configuration
                            format!(" show {}", parts.get(1).unwrap_or(&""))
                        }
                        _ => format!(" {}", args),
                    }
                },
            },
        );

        m.insert(
            "net",
            CommandMapping {
                target_cmd: "systemctl",
                description: "Network/service commands",
                arg_translator: |args| {
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    if parts.is_empty() {
                        return String::new();
                    }

                    match parts[0].to_lowercase().as_str() {
                        "start" => format!(" start {}", parts.get(1).unwrap_or(&"")),
                        "stop" => format!(" stop {}", parts.get(1).unwrap_or(&"")),
                        "user" => {
                            // net user -> getent passwd
                            String::new()
                        }
                        "use" => {
                            // net use -> mount (requires different syntax)
                            String::new()
                        }
                        "view" => {
                            // net view -> requires smbclient
                            String::new()
                        }
                        _ => format!(" {}", args),
                    }
                },
            },
        );

        m
    });

impl CommandTranslator {
    /// Create a new command translator
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        let current_os = Self::detect_os();

        Self {
            enabled,
            current_os,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Detect the current operating system
    fn detect_os() -> OsType {
        if cfg!(target_os = "windows") {
            OsType::Windows
        } else if cfg!(target_os = "linux") {
            OsType::Linux
        } else if cfg!(target_os = "macos") {
            OsType::MacOs
        } else {
            OsType::Unknown
        }
    }

    /// Translate a command if translation is enabled and applicable
    /// Supports pipelining with |, >, >>, <, &&, ||, ;
    #[must_use]
    pub fn translate(&self, command: &str) -> TranslationResult {
        let command = command.trim();
        let mut errors: Vec<TranslationError> = Vec::new();

        // Fast path: disabled or empty command
        if !self.enabled || command.is_empty() {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors,
                has_pipeline: false,
            };
        }

        // Check if command contains pipeline operators
        let has_pipeline = self.contains_pipeline_operators(command);

        if has_pipeline {
            // Handle pipelined commands
            return self.translate_pipeline(command);
        }

        // Single command translation
        self.translate_single_command(command, &mut errors)
    }

    /// Check if a command contains any pipeline operators
    fn contains_pipeline_operators(&self, command: &str) -> bool {
        // Check for operators while avoiding false positives in strings
        // This is a simplified check - full parsing would require proper tokenization
        let operators = ["||", "&&", "|", ">>", ">", "<", ";"];
        
        for op in operators {
            if command.contains(op) {
                // Make sure it's not inside quotes
                let mut in_single_quote = false;
                let mut in_double_quote = false;
                let mut pos = 0;
                
                for c in command.chars() {
                    match c {
                        '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                        '"' if !in_single_quote => in_double_quote = !in_double_quote,
                        _ => {}
                    }
                    
                    if !in_single_quote && !in_double_quote {
                        // Check if we're at an operator
                        let remaining = &command[pos..];
                        if remaining.starts_with(op) {
                            return true;
                        }
                    }
                    pos += c.len_utf8();
                }
            }
        }
        false
    }

    /// Parse a command line into pipeline segments
    fn parse_pipeline(&self, command: &str) -> Vec<PipelineSegment> {
        let mut segments = Vec::new();
        let mut current_segment = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut chars = command.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                    current_segment.push(c);
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                    current_segment.push(c);
                }
                '|' if !in_single_quote && !in_double_quote => {
                    // Check for || vs |
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        segments.push(PipelineSegment {
                            command: current_segment.trim().to_string(),
                            operator: Some(PipelineOperator::Or),
                        });
                    } else {
                        segments.push(PipelineSegment {
                            command: current_segment.trim().to_string(),
                            operator: Some(PipelineOperator::Pipe),
                        });
                    }
                    current_segment = String::new();
                }
                '&' if !in_single_quote && !in_double_quote => {
                    // Check for &&
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        segments.push(PipelineSegment {
                            command: current_segment.trim().to_string(),
                            operator: Some(PipelineOperator::And),
                        });
                        current_segment = String::new();
                    } else {
                        // Single & (background) - just pass through
                        current_segment.push(c);
                    }
                }
                '>' if !in_single_quote && !in_double_quote => {
                    // Check for >> vs >
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        segments.push(PipelineSegment {
                            command: current_segment.trim().to_string(),
                            operator: Some(PipelineOperator::RedirectAppend),
                        });
                    } else {
                        segments.push(PipelineSegment {
                            command: current_segment.trim().to_string(),
                            operator: Some(PipelineOperator::RedirectOut),
                        });
                    }
                    current_segment = String::new();
                }
                '<' if !in_single_quote && !in_double_quote => {
                    segments.push(PipelineSegment {
                        command: current_segment.trim().to_string(),
                        operator: Some(PipelineOperator::RedirectIn),
                    });
                    current_segment = String::new();
                }
                ';' if !in_single_quote && !in_double_quote => {
                    segments.push(PipelineSegment {
                        command: current_segment.trim().to_string(),
                        operator: Some(PipelineOperator::Semicolon),
                    });
                    current_segment = String::new();
                }
                _ => {
                    current_segment.push(c);
                }
            }
        }

        // Add the last segment
        if !current_segment.trim().is_empty() {
            segments.push(PipelineSegment {
                command: current_segment.trim().to_string(),
                operator: None,
            });
        }

        segments
    }

    /// Translate a pipeline command (command with operators like |, >, &&, etc.)
    fn translate_pipeline(&self, command: &str) -> TranslationResult {
        let segments = self.parse_pipeline(command);
        let mut errors: Vec<TranslationError> = Vec::new();
        let mut translated_parts: Vec<String> = Vec::new();
        let mut any_translated = false;
        let mut descriptions: Vec<String> = Vec::new();

        for segment in &segments {
            if segment.command.is_empty() {
                // Handle empty segments (e.g., leading operator)
                if let Some(op) = &segment.operator {
                    translated_parts.push(op.as_str().to_string());
                }
                continue;
            }

            // Translate the command part
            let result = self.translate_single_command(&segment.command, &mut errors);
            
            if result.translated {
                any_translated = true;
                translated_parts.push(result.final_command);
                if !result.description.is_empty() {
                    descriptions.push(result.description);
                }
            } else {
                // Keep original if not translated
                translated_parts.push(segment.command.clone());
            }

            // Add the operator
            if let Some(op) = &segment.operator {
                translated_parts.push(format!(" {} ", op.as_str()));
            }
        }

        // Join all parts
        let final_command = translated_parts.join("").trim().to_string();
        let description = if descriptions.is_empty() {
            String::new()
        } else {
            descriptions.join("; ")
        };

        TranslationResult {
            translated: any_translated,
            original_command: command.to_string(),
            final_command,
            description,
            errors,
            has_pipeline: true,
        }
    }

    /// Translate a single command (no pipeline operators)
    fn translate_single_command(&self, command: &str, errors: &mut Vec<TranslationError>) -> TranslationResult {
        let command = command.trim();

        if command.is_empty() {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors: Vec::new(),
                has_pipeline: false,
            };
        }

        // Parse command into parts - avoid collecting into Vec
        let mut parts = command.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => {
                errors.push(TranslationError::InvalidSyntax("Empty command".to_string()));
                return TranslationResult {
                    translated: false,
                    original_command: command.to_string(),
                    final_command: command.to_string(),
                    description: String::new(),
                    errors: Vec::new(),
                    has_pipeline: false,
                };
            }
        };

        let args = command.strip_prefix(cmd).unwrap_or("").trim();

        // Determine which direction to translate
        let (mapping, should_translate) = match self.current_os {
            OsType::Windows => {
                // On Windows, translate Linux commands to Windows
                (LINUX_TO_WINDOWS_MAP.get(cmd), true)
            }
            OsType::Linux | OsType::MacOs => {
                // On Linux/Mac, translate Windows commands to Linux
                (WINDOWS_TO_LINUX_MAP.get(cmd), true)
            }
            OsType::Unknown => (None, false),
        };

        if !should_translate {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors: Vec::new(),
                has_pipeline: false,
            };
        }

        // Don't translate cd command with arguments on any platform
        if cmd == "cd" && !args.is_empty() {
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors: Vec::new(),
                has_pipeline: false,
            };
        }

        // Special case: translate bare "cd" to "pwd" on Windows (shows current directory)
        if cmd == "cd" && self.current_os == OsType::Windows && args.is_empty() {
            // On Windows, bare "cd" shows current directory like pwd
            // Let it through for translation
        } else if cmd == "cd" && args.is_empty() {
            // On Linux/Mac, bare "cd" changes to home directory, not pwd
            return TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors: Vec::new(),
                has_pipeline: false,
            };
        }

        if let Some(mapping) = mapping {
            let translated_args = (mapping.arg_translator)(args);
            // Use String::with_capacity for more efficient concatenation
            let mut final_cmd =
                String::with_capacity(mapping.target_cmd.len() + translated_args.len());
            final_cmd.push_str(mapping.target_cmd);
            final_cmd.push_str(&translated_args);

            TranslationResult {
                translated: true,
                original_command: command.to_string(),
                final_command: final_cmd.trim().to_string(),
                description: mapping.description.to_string(),
                errors: Vec::new(),
                has_pipeline: false,
            }
        } else {
            // Command not found in translation map - add error for context
            errors.push(TranslationError::UnknownCommand(cmd.to_string()));
            TranslationResult {
                translated: false,
                original_command: command.to_string(),
                final_command: command.to_string(),
                description: String::new(),
                errors: Vec::new(),
                has_pipeline: false,
            }
        }
    }

    /// Enable or disable command translation
    #[allow(dead_code)] // Public API
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if translation is enabled
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current OS type
    #[allow(dead_code)] // Public API
    #[must_use]
    pub fn current_os(&self) -> OsType {
        self.current_os
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Basic Translator Tests ==========

    #[test]
    fn test_translator_creation() {
        let translator = CommandTranslator::new(true);
        assert!(translator.is_enabled());
    }

    #[test]
    fn test_os_detection() {
        let translator = CommandTranslator::new(true);
        let os = translator.current_os();

        #[cfg(target_os = "windows")]
        assert_eq!(os, OsType::Windows);

        #[cfg(target_os = "linux")]
        assert_eq!(os, OsType::Linux);

        #[cfg(target_os = "macos")]
        assert_eq!(os, OsType::MacOs);
    }

    #[test]
    fn test_disabled_translator() {
        let translator = CommandTranslator::new(false);

        let result = translator.translate("ls");
        assert!(!result.translated);
        assert_eq!(result.original_command, "ls");
        assert_eq!(result.final_command, "ls");
    }

    #[test]
    fn test_unknown_command() {
        let translator = CommandTranslator::new(true);

        let result = translator.translate("unknown_command");
        assert!(!result.translated);
        assert_eq!(result.original_command, "unknown_command");
        assert_eq!(result.final_command, "unknown_command");
    }

    #[test]
    fn test_empty_command() {
        let translator = CommandTranslator::new(true);

        let result = translator.translate("");
        assert!(!result.translated);
        assert_eq!(result.final_command, "");
    }

    // ========== Windows->Linux Basic Tests ==========

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_basic() {
        let translator = CommandTranslator::new(true);

        let result = translator.translate("dir");
        assert!(result.translated);
        assert_eq!(result.final_command, "ls");

        let result = translator.translate("cls");
        assert!(result.translated);
        assert_eq!(result.final_command, "clear");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_file_ops() {
        let translator = CommandTranslator::new(true);

        // type -> cat
        let result = translator.translate("type file.txt");
        assert!(result.translated);
        assert_eq!(result.final_command, "cat file.txt");

        // copy -> cp
        let result = translator.translate("copy src.txt dst.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("cp"));
        assert!(result.final_command.contains("src.txt"));
        assert!(result.final_command.contains("dst.txt"));

        // move -> mv
        let result = translator.translate("move old.txt new.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("mv"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_del_flags() {
        let translator = CommandTranslator::new(true);

        // del /S -> rm -r
        let result = translator.translate("del /S folder");
        assert!(result.translated);
        assert!(result.final_command.contains("rm"));
        assert!(result.final_command.contains("-r"));
        assert!(result.final_command.contains("folder"));

        // del /F /Q -> rm -f
        let result = translator.translate("del /F /Q file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("rm"));
        assert!(result.final_command.contains("-f"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_dir_flags() {
        let translator = CommandTranslator::new(true);

        // dir /A -> ls -a
        let result = translator.translate("dir /A");
        assert!(result.translated);
        assert!(result.final_command.contains("ls"));
        assert!(result.final_command.contains("-a"));

        // dir /S -> ls -R
        let result = translator.translate("dir /S");
        assert!(result.translated);
        assert!(result.final_command.contains("-R"));

        // dir /B -> ls -1
        let result = translator.translate("dir /B");
        assert!(result.translated);
        assert!(result.final_command.contains("-1"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_network() {
        let translator = CommandTranslator::new(true);

        // tracert -> traceroute
        let result = translator.translate("tracert google.com");
        assert!(result.translated);
        assert!(result.final_command.contains("traceroute"));

        // ipconfig -> ip addr
        let result = translator.translate("ipconfig");
        assert!(result.translated);
        assert!(result.final_command.contains("ip addr"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_findstr() {
        let translator = CommandTranslator::new(true);

        // findstr /I -> grep -i
        let result = translator.translate("findstr /I \"pattern\" file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("grep"));
        assert!(result.final_command.contains("-i"));

        // findstr /N -> grep -n
        let result = translator.translate("findstr /N \"search\" log.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("-n"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_taskkill() {
        let translator = CommandTranslator::new(true);

        // taskkill /F /PID 1234 -> kill -9 1234
        let result = translator.translate("taskkill /F /PID 1234");
        assert!(result.translated);
        assert!(result.final_command.contains("kill"));
        assert!(result.final_command.contains("-9"));
        assert!(result.final_command.contains("1234"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_ping() {
        let translator = CommandTranslator::new(true);

        // ping -n 4 host -> ping -c 4 host
        let result = translator.translate("ping -n 4 google.com");
        assert!(result.translated);
        assert!(result.final_command.contains("ping"));
        assert!(result.final_command.contains("-c 4"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_windows_to_linux_mkdir() {
        let translator = CommandTranslator::new(true);

        // md -> mkdir -p
        let result = translator.translate("md newfolder");
        assert!(result.translated);
        assert!(result.final_command.contains("mkdir"));
        assert!(result.final_command.contains("-p"));
        assert!(result.final_command.contains("newfolder"));
    }

    // ========== Linux->Windows Basic Tests ==========

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_basic() {
        let translator = CommandTranslator::new(true);

        let result = translator.translate("ls");
        assert!(result.translated);
        assert_eq!(result.final_command, "dir");

        let result = translator.translate("pwd");
        assert!(result.translated);
        assert_eq!(result.final_command, "cd");

        let result = translator.translate("clear");
        assert!(result.translated);
        assert_eq!(result.final_command, "cls");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_ls_flags() {
        let translator = CommandTranslator::new(true);

        // ls -a -> dir /A
        let result = translator.translate("ls -a");
        assert!(result.translated);
        assert!(result.final_command.contains("dir"));
        assert!(result.final_command.contains("/A"));

        // ls -R -> dir /S
        let result = translator.translate("ls -R");
        assert!(result.translated);
        assert!(result.final_command.contains("/S"));

        // ls -1 -> dir /B
        let result = translator.translate("ls -1");
        assert!(result.translated);
        assert!(result.final_command.contains("/B"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_cat() {
        let translator = CommandTranslator::new(true);

        let result = translator.translate("cat file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("type"));
        assert!(result.final_command.contains("file.txt"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_rm_flags() {
        let translator = CommandTranslator::new(true);

        // rm -r -> del /S
        let result = translator.translate("rm -r folder");
        assert!(result.translated);
        assert!(result.final_command.contains("del"));
        assert!(result.final_command.contains("/S"));

        // rm -f -> del /F /Q
        let result = translator.translate("rm -f file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("/F"));
        assert!(result.final_command.contains("/Q"));

        // rm -rf (combined flags)
        let result = translator.translate("rm -rf folder");
        assert!(result.translated);
        assert!(result.final_command.contains("/S"));
        assert!(result.final_command.contains("/F"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_grep() {
        let translator = CommandTranslator::new(true);

        // grep -i -> findstr /I
        let result = translator.translate("grep -i \"pattern\" file.txt");
        assert!(result.translated);
        assert!(result.final_command.contains("findstr"));
        assert!(result.final_command.contains("/I"));

        // grep -r -> findstr /S
        let result = translator.translate("grep -r \"search\" .");
        assert!(result.translated);
        assert!(result.final_command.contains("/S"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_ping() {
        let translator = CommandTranslator::new(true);

        // ping -c 4 host -> ping -n 4 host
        let result = translator.translate("ping -c 4 google.com");
        assert!(result.translated);
        assert!(result.final_command.contains("ping"));
        assert!(result.final_command.contains("-n 4"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_linux_to_windows_kill() {
        let translator = CommandTranslator::new(true);

        // kill -9 1234 -> taskkill /F /PID 1234
        let result = translator.translate("kill -9 1234");
        assert!(result.translated);
        assert!(result.final_command.contains("taskkill"));
        assert!(result.final_command.contains("/F"));
        assert!(result.final_command.contains("/PID"));
        assert!(result.final_command.contains("1234"));
    }

    // ========== Helper Function Tests ==========

    #[test]
    fn test_has_flag() {
        assert!(has_flag("-a", 'a', None));
        assert!(has_flag("-la", 'a', None));
        assert!(has_flag("-la", 'l', None));
        assert!(has_flag("--all", 'a', Some("all")));
        assert!(has_flag("-rf folder", 'r', None));
        assert!(has_flag("-rf folder", 'f', None));
        assert!(!has_flag("-la", 'x', None));
        assert!(!has_flag("--verbose", 'v', Some("all")));
    }

    #[test]
    fn test_extract_paths() {
        let paths = extract_paths("-a /home/user");
        assert_eq!(paths, vec!["/home/user"]);

        let paths = extract_paths("-rf folder1 folder2");
        assert_eq!(paths, vec!["folder1", "folder2"]);

        let paths = extract_paths("file.txt");
        assert_eq!(paths, vec!["file.txt"]);
    }

    #[test]
    fn test_get_flag_value() {
        assert_eq!(get_flag_value("-n 10 file.txt", 'n', None), Some("10"));
        assert_eq!(get_flag_value("--lines=5", 'n', Some("lines")), Some("5"));
        assert_eq!(get_flag_value("-c 4 host", 'c', None), Some("4"));
        assert_eq!(get_flag_value("-a file.txt", 'n', None), None);
    }

    // ========== Argument Translator Tests ==========

    #[test]
    fn test_ls_to_dir_args() {
        // -a flag
        let result = ls_to_dir_args("-a");
        assert!(result.contains("/A"));

        // -R flag
        let result = ls_to_dir_args("-R");
        assert!(result.contains("/S"));

        // -1 flag
        let result = ls_to_dir_args("-1");
        assert!(result.contains("/B"));

        // Combined flags
        let result = ls_to_dir_args("-laR");
        assert!(result.contains("/A"));
        assert!(result.contains("/S"));

        // With path
        let result = ls_to_dir_args("-la /home");
        assert!(result.contains("/A"));
        assert!(result.contains("/home"));
    }

    #[test]
    fn test_dir_to_ls_args() {
        // /A flag
        let result = dir_to_ls_args("/A");
        assert!(result.contains("-a"));

        // /S flag
        let result = dir_to_ls_args("/S");
        assert!(result.contains("-R"));

        // /B flag
        let result = dir_to_ls_args("/B");
        assert!(result.contains("-1"));

        // Case insensitive
        let result = dir_to_ls_args("/a");
        assert!(result.contains("-a"));
    }

    #[test]
    fn test_rm_to_del_args() {
        // -r flag
        let result = rm_to_del_args("-r folder");
        assert!(result.contains("/S"));
        assert!(result.contains("folder"));

        // -f flag
        let result = rm_to_del_args("-f file.txt");
        assert!(result.contains("/F"));
        assert!(result.contains("/Q"));

        // Combined -rf
        let result = rm_to_del_args("-rf folder");
        assert!(result.contains("/S"));
        assert!(result.contains("/F"));
    }

    #[test]
    fn test_del_to_rm_args() {
        // /S flag
        let result = del_to_rm_args("/S folder");
        assert!(result.contains("-r"));

        // /F flag
        let result = del_to_rm_args("/F file.txt");
        assert!(result.contains("-f"));

        // /P flag
        let result = del_to_rm_args("/P file.txt");
        assert!(result.contains("-i"));
    }

    #[test]
    fn test_grep_to_findstr_args() {
        // -i flag
        let result = grep_to_findstr_args("-i \"pattern\" file.txt");
        assert!(result.contains("/I"));

        // -n flag
        let result = grep_to_findstr_args("-n \"pattern\" file.txt");
        assert!(result.contains("/N"));

        // -r flag
        let result = grep_to_findstr_args("-r \"pattern\" .");
        assert!(result.contains("/S"));
    }

    #[test]
    fn test_findstr_to_grep_args() {
        // /I flag
        let result = findstr_to_grep_args("/I \"pattern\" file.txt");
        assert!(result.contains("-i"));

        // /N flag
        let result = findstr_to_grep_args("/N \"pattern\" file.txt");
        assert!(result.contains("-n"));

        // /S flag
        let result = findstr_to_grep_args("/S \"pattern\"");
        assert!(result.contains("-r"));
    }

    #[test]
    fn test_ping_args_translation() {
        // Linux -c to Windows -n
        let result = ping_linux_to_windows_args("-c 4 google.com");
        assert!(result.contains("-n 4"));
        assert!(result.contains("google.com"));

        // Windows -n to Linux -c
        let result = ping_windows_to_linux_args("-n 4 google.com");
        assert!(result.contains("-c 4"));
    }

    #[test]
    fn test_kill_to_taskkill_args() {
        // -9 flag
        let result = kill_to_taskkill_args("-9 1234");
        assert!(result.contains("/F"));
        assert!(result.contains("/PID"));
        assert!(result.contains("1234"));

        // Without -9
        let result = kill_to_taskkill_args("5678");
        assert!(result.contains("/PID"));
        assert!(result.contains("5678"));
        assert!(!result.contains("/F"));
    }

    #[test]
    fn test_taskkill_to_kill_args() {
        // /F /PID flags
        let result = taskkill_to_kill_args("/F /PID 1234");
        assert!(result.contains("-9"));
        assert!(result.contains("1234"));

        // Without /F
        let result = taskkill_to_kill_args("/PID 5678");
        assert!(result.contains("5678"));
        assert!(!result.contains("-9"));
    }

    #[test]
    fn test_sort_args_translation() {
        // Linux -r to Windows /R
        let result = sort_linux_to_windows_args("-r file.txt");
        assert!(result.contains("/R"));
        assert!(result.contains("file.txt"));

        // Windows /R to Linux -r
        let result = sort_windows_to_linux_args("/R file.txt");
        assert!(result.contains("-r"));
    }

    #[test]
    fn test_head_to_ps_args() {
        // Default (no -n flag)
        let result = head_to_ps_args("file.txt");
        assert!(result.contains("-Head 10"));
        assert!(result.contains("file.txt"));

        // With -n flag
        let result = head_to_ps_args("-n 20 file.txt");
        assert!(result.contains("-Head 20"));
    }

    #[test]
    fn test_tail_to_ps_args() {
        // Default (no -n flag)
        let result = tail_to_ps_args("file.txt");
        assert!(result.contains("-Tail 10"));

        // With -f flag
        let result = tail_to_ps_args("-f file.txt");
        assert!(result.contains("-Wait"));
    }

    // ========== Command Mapping Coverage Tests ==========

    #[test]
    fn test_linux_to_windows_map_coverage() {
        // Verify essential commands are in the map
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("ls"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("cat"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("rm"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("cp"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("mv"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("mkdir"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("rmdir"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("grep"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("head"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("tail"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("ping"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("curl"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("wget"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("ps"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("kill"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("killall"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("df"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("du"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("tar"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("zip"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("unzip"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("find"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("sort"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("netstat"));
        assert!(LINUX_TO_WINDOWS_MAP.contains_key("traceroute"));
    }

    #[test]
    fn test_windows_to_linux_map_coverage() {
        // Verify essential commands are in the map
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("dir"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("type"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("del"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("copy"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("move"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("md"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("rd"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("findstr"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("find"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("ping"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("ipconfig"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("tasklist"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("taskkill"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("xcopy"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("sort"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("netstat"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("tracert"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("cls"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("where"));
        assert!(WINDOWS_TO_LINUX_MAP.contains_key("hostname"));
    }

    #[test]
    fn test_identity_args() {
        assert_eq!(identity_args("test"), "test");
        assert_eq!(identity_args(""), "");
        assert_eq!(identity_args("multiple args here"), "multiple args here");
    }

    // ========== Pipeline Tests ==========

    #[test]
    fn test_pipeline_detection() {
        let translator = CommandTranslator::new(true);

        // Commands with pipes
        assert!(translator.contains_pipeline_operators("ls | grep foo"));
        assert!(translator.contains_pipeline_operators("cat file.txt | sort | uniq"));

        // Commands with redirects
        assert!(translator.contains_pipeline_operators("echo hello > file.txt"));
        assert!(translator.contains_pipeline_operators("cat < input.txt"));
        assert!(translator.contains_pipeline_operators("echo hello >> file.txt"));

        // Commands with chaining
        assert!(translator.contains_pipeline_operators("ls && echo done"));
        assert!(translator.contains_pipeline_operators("ls || echo failed"));
        assert!(translator.contains_pipeline_operators("ls; pwd"));

        // Commands without pipelines
        assert!(!translator.contains_pipeline_operators("ls -la"));
        assert!(!translator.contains_pipeline_operators("cat file.txt"));
    }

    #[test]
    fn test_pipeline_parsing() {
        let translator = CommandTranslator::new(true);

        // Single pipe
        let segments = translator.parse_pipeline("ls | grep foo");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].command, "ls");
        assert_eq!(segments[0].operator, Some(PipelineOperator::Pipe));
        assert_eq!(segments[1].command, "grep foo");
        assert!(segments[1].operator.is_none());

        // Multiple pipes
        let segments = translator.parse_pipeline("cat file.txt | sort | uniq");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].command, "cat file.txt");
        assert_eq!(segments[1].command, "sort");
        assert_eq!(segments[2].command, "uniq");

        // Output redirect
        let segments = translator.parse_pipeline("echo hello > output.txt");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].command, "echo hello");
        assert_eq!(segments[0].operator, Some(PipelineOperator::RedirectOut));
        assert_eq!(segments[1].command, "output.txt");

        // Command chaining with &&
        let segments = translator.parse_pipeline("ls && echo done");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].command, "ls");
        assert_eq!(segments[0].operator, Some(PipelineOperator::And));
        assert_eq!(segments[1].command, "echo done");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_pipeline_translation_linux() {
        let translator = CommandTranslator::new(true);

        // dir | findstr -> ls | grep
        let result = translator.translate("dir | findstr foo");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("ls"));
        assert!(result.final_command.contains("|"));
        assert!(result.final_command.contains("grep"));

        // type file.txt | sort -> cat file.txt | sort
        let result = translator.translate("type file.txt | sort");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("cat"));

        // Command chaining: cls && dir -> clear && ls
        let result = translator.translate("cls && dir");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("clear"));
        assert!(result.final_command.contains("&&"));
        assert!(result.final_command.contains("ls"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_pipeline_translation_windows() {
        let translator = CommandTranslator::new(true);

        // ls | grep -> dir | findstr
        let result = translator.translate("ls | grep foo");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("dir"));
        assert!(result.final_command.contains("|"));
        assert!(result.final_command.contains("findstr"));

        // cat file.txt | sort -> type file.txt | sort
        let result = translator.translate("cat file.txt | sort");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("type"));

        // Command chaining: clear && ls -> cls && dir
        let result = translator.translate("clear && ls");
        assert!(result.translated);
        assert!(result.has_pipeline);
        assert!(result.final_command.contains("cls"));
        assert!(result.final_command.contains("&&"));
        assert!(result.final_command.contains("dir"));
    }

    #[test]
    fn test_pipeline_with_redirect() {
        let translator = CommandTranslator::new(true);

        // Output redirect should be preserved
        let result = translator.translate("echo hello > output.txt");
        assert!(result.has_pipeline);
        assert!(result.final_command.contains(">"));
        assert!(result.final_command.contains("output.txt"));

        // Append redirect
        let result = translator.translate("echo hello >> output.txt");
        assert!(result.has_pipeline);
        assert!(result.final_command.contains(">>"));
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_translation_error_display() {
        let err = TranslationError::UnknownCommand("foo".to_string());
        assert!(format!("{}", err).contains("Unknown command"));
        assert!(format!("{}", err).contains("foo"));

        let err = TranslationError::InvalidSyntax("bad syntax".to_string());
        assert!(format!("{}", err).contains("Invalid syntax"));

        let err = TranslationError::UnsupportedOperator(">>>".to_string());
        assert!(format!("{}", err).contains("Unsupported operator"));

        let err = TranslationError::PartialTranslation("some parts failed".to_string());
        assert!(format!("{}", err).contains("Partial translation"));
    }

    #[test]
    fn test_translation_result_errors() {
        let translator = CommandTranslator::new(true);

        // Unknown command should have no errors (just not translated)
        let result = translator.translate("unknowncommand");
        assert!(!result.translated);
        // The command is passed through, not an error

        // Valid translation should have no errors
        #[cfg(not(target_os = "windows"))]
        {
            let result = translator.translate("dir");
            assert!(result.translated);
            assert!(result.errors.is_empty());
        }

        #[cfg(target_os = "windows")]
        {
            let result = translator.translate("ls");
            assert!(result.translated);
            assert!(result.errors.is_empty());
        }
    }

    #[test]
    fn test_pipeline_operator_from_str() {
        assert_eq!(PipelineOperator::from_str("|"), Some(PipelineOperator::Pipe));
        assert_eq!(PipelineOperator::from_str(">"), Some(PipelineOperator::RedirectOut));
        assert_eq!(PipelineOperator::from_str(">>"), Some(PipelineOperator::RedirectAppend));
        assert_eq!(PipelineOperator::from_str("<"), Some(PipelineOperator::RedirectIn));
        assert_eq!(PipelineOperator::from_str("&&"), Some(PipelineOperator::And));
        assert_eq!(PipelineOperator::from_str("||"), Some(PipelineOperator::Or));
        assert_eq!(PipelineOperator::from_str(";"), Some(PipelineOperator::Semicolon));
        assert_eq!(PipelineOperator::from_str("invalid"), None);
    }

    #[test]
    fn test_pipeline_operator_as_str() {
        assert_eq!(PipelineOperator::Pipe.as_str(), "|");
        assert_eq!(PipelineOperator::RedirectOut.as_str(), ">");
        assert_eq!(PipelineOperator::RedirectAppend.as_str(), ">>");
        assert_eq!(PipelineOperator::RedirectIn.as_str(), "<");
        assert_eq!(PipelineOperator::And.as_str(), "&&");
        assert_eq!(PipelineOperator::Or.as_str(), "||");
        assert_eq!(PipelineOperator::Semicolon.as_str(), ";");
    }

    #[test]
    fn test_complex_pipeline() {
        let translator = CommandTranslator::new(true);

        // Complex pipeline with multiple operators
        let result = translator.translate("echo start; ls | grep test && echo done || echo failed");
        assert!(result.has_pipeline);
        // The command should be processed
        assert!(!result.final_command.is_empty());
    }

    #[test]
    fn test_pipeline_with_quotes() {
        let translator = CommandTranslator::new(true);

        // Operators inside quotes should not be treated as pipeline operators
        // Note: This is a simplified test - full quote handling is complex
        let segments = translator.parse_pipeline("echo 'hello world'");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].command, "echo 'hello world'");
    }
}
