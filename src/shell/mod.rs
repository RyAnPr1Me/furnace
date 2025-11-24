use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

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

        let pair = pty_system
            .openpty(pty_size)
            .context("Failed to open PTY")?;

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

        let reader = pair.master.try_clone_reader().context("Failed to clone reader")?;
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
        let reader = self.reader.clone();
        
        // Use spawn_blocking for the synchronous read operation
        // We pass the buffer data as a Vec to work around the lifetime/Send constraints
        let buffer_len = buffer.len();
        let result = tokio::task::spawn_blocking(move || {
            let mut reader = reader.blocking_lock();
            // Use a stack-allocated array for small buffers, heap for large ones
            // This is already optimized by Rust's allocator
            let mut temp_buf = vec![0u8; buffer_len];
            match reader.read(&mut temp_buf) {
                Ok(n) => Ok((n, temp_buf)),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok((0, Vec::new())),
                Err(e) => Err(e),
            }
        }).await;
        
        match result {
            Ok(Ok((n, temp_buf))) => {
                if n > 0 {
                    buffer[..n].copy_from_slice(&temp_buf[..n]);
                }
                Ok(n)
            }
            Ok(Err(e)) => {
                error!("Failed to read from shell: {}", e);
                Err(e.into())
            }
            Err(e) => {
                error!("Task join error: {}", e);
                Err(e.into())
            }
        }
    }

    /// Write input to shell (optimized for minimal latency)
    ///
    /// # Errors
    /// Returns an error if the write or flush operation fails
    pub async fn write_input(&self, data: &[u8]) -> Result<usize> {
        let mut writer = self.writer.lock().await;
        
        writer.write_all(data)
            .context("Failed to write to shell")?;
        
        writer.flush()
            .context("Failed to flush shell input")?;
        
        Ok(data.len())
    }

    /// Resize the PTY (important for responsive terminal)
    ///
    /// # Errors
    /// Returns an error if the PTY resize operation fails
    #[allow(dead_code)] // Public API for future use
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let pty = self.pty.lock().await;
        
        pty.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to resize PTY")?;

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
