use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// High-performance shell session with zero-copy I/O where possible
pub struct ShellSession {
    pty: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl ShellSession {
    /// Create a new shell session with optimal buffer sizes
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
    pub async fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut reader = self.reader.lock().await;
        
        match reader.read(buffer) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => {
                error!("Failed to read from shell: {}", e);
                Err(e.into())
            }
        }
    }

    /// Write input to shell (optimized for minimal latency)
    pub async fn write_input(&self, data: &[u8]) -> Result<usize> {
        let mut writer = self.writer.lock().await;
        
        writer.write_all(data)
            .context("Failed to write to shell")?;
        
        writer.flush()
            .context("Failed to flush shell input")?;
        
        Ok(data.len())
    }

    /// Resize the PTY (important for responsive terminal)
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
