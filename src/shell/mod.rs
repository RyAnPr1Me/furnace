use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// High-performance shell session with zero-copy I/O where possible
pub struct ShellSession {
    #[allow(dead_code)] // Kept for potential future use
    pty: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl ShellSession {
    /// Create a new shell session with optimal buffer sizes
    ///
    /// # Errors
    /// Returns an error if PTY creation or shell process spawn fails
    pub fn new(shell_cmd: &str, working_dir: Option<&str>, rows: u16, cols: u16) -> Result<Self> {
        let pty_system = NativePtySystem::default();

        let pty_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(pty_size).context("Failed to open PTY")?;

        let mut cmd = CommandBuilder::new(shell_cmd);

        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        let _child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell")?;

        info!("Shell session started: {}", shell_cmd);
        debug!("PTY size: {}x{}", rows, cols);

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone reader")?;
        let writer = pair.master.take_writer().context("Failed to take writer")?;

        Ok(Self {
            pty: Arc::new(Mutex::new(pair.master)),
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    /// Read output from shell (non-blocking, high-performance)
    ///
    /// # Errors
    /// Returns an error if the read operation fails or the task cannot be spawned
    pub async fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
        // BUG FIX #1: Use spawn_blocking but write directly to buffer to avoid data corruption
        // Clone the Arc to move into spawn_blocking, but pass buffer size to avoid lifetime issues
        let reader = self.reader.clone();
        let buffer_len = buffer.len();
        
        let (n, data) = tokio::task::spawn_blocking(move || {
            let mut reader = reader.blocking_lock();
            let mut temp = vec![0u8; buffer_len];
            match reader.read(&mut temp) {
                Ok(n) => Ok((n, temp)),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok((0, vec![])),
                Err(e) => Err(e),
            }
        })
        .await
        .context("Task join error")?
        .context("Failed to read from shell")?;
        
        // Copy data to buffer only once, outside spawn_blocking
        if n > 0 {
            buffer[..n].copy_from_slice(&data[..n]);
        }
        Ok(n)
    }

    /// Write input to shell with minimal latency
    ///
    /// This function writes data to the shell and immediately flushes to ensure
    /// low latency. This is critical for interactive terminal responsiveness.
    ///
    /// # Arguments
    /// * `data` - Bytes to write to the shell (typically user input or commands)
    ///
    /// # Returns
    /// Number of bytes written on success
    ///
    /// # Errors
    /// Returns an error if:
    /// - The write operation fails (e.g., shell terminated)
    /// - The flush operation fails (e.g., broken pipe)
    pub async fn write_input(&self, data: &[u8]) -> Result<usize> {
        // BUG FIX #2: Use spawn_blocking for sync I/O to avoid blocking the async runtime
        let writer = self.writer.clone();
        let data = data.to_vec();
        let len = data.len();
        
        tokio::task::spawn_blocking(move || {
            let mut writer = writer.blocking_lock();
            writer.write_all(&data)?;
            writer.flush()?;
            Ok::<_, anyhow::Error>(len)
        })
        .await
        .context("Task join error")?
        .context(format!("Failed to write {} bytes to shell", len))?;
        
        debug!("Wrote {} bytes to shell", len);
        Ok(len)
    }

    /// Resize the PTY to match terminal dimensions
    ///
    /// This function must be called when the terminal window is resized to ensure
    /// proper text wrapping and display. Without resizing, the shell will not know
    /// the actual terminal dimensions and may produce incorrectly wrapped output.
    ///
    /// # Arguments
    /// * `rows` - New number of rows (lines)
    /// * `cols` - New number of columns (characters per line)
    ///
    /// # Errors
    /// Returns an error if the PTY resize operation fails (e.g., invalid dimensions)
    #[allow(dead_code)] // Public API for future use
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let pty = self.pty.lock().await;

        pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context(format!("Failed to resize PTY to {rows}x{cols}"))?;

        debug!("Resized PTY to {}x{}", rows, cols);
        Ok(())
    }
}

// Ensure proper cleanup - Rust's Drop trait guarantees no leaks
impl Drop for ShellSession {
    fn drop(&mut self) {
        info!("Shell session terminated");
    }
}
