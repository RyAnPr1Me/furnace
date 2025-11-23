use sysinfo::System;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// System resource monitor for displaying resource usage
pub struct ResourceMonitor {
    system: Arc<Mutex<System>>,
    last_update: Instant,
    update_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub cpu_usage: f32,
    pub cpu_count: usize,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
    pub process_count: usize,
    pub network_rx: u64,
    pub network_tx: u64,
    pub disk_usage: Vec<DiskInfo>,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub used: u64,
    pub total: u64,
    pub percent: f32,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system: Arc::new(Mutex::new(system)),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500), // Update every 500ms
        }
    }

    /// Get current resource statistics
    pub fn get_stats(&mut self) -> ResourceStats {
        // Only update if enough time has passed
        if self.last_update.elapsed() >= self.update_interval {
            if let Ok(mut system) = self.system.lock() {
                system.refresh_cpu();
                system.refresh_memory();
                system.refresh_processes();
            }
            self.last_update = Instant::now();
        }

        let system = self.system.lock().unwrap();

        // CPU usage (average across all cores)
        let cpu_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() 
            / system.cpus().len().max(1) as f32;
        
        // Memory usage
        let memory_used = system.used_memory();
        let memory_total = system.total_memory();
        let memory_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Process count
        let process_count = system.processes().len();

        // Network and disk stats not implemented yet (API compatibility varies by platform)
        // These fields are reserved for future implementation when stable cross-platform APIs are available
        let network_rx = 0u64;
        let network_tx = 0u64;
        let disk_usage = Vec::new();

        ResourceStats {
            cpu_usage,
            cpu_count: system.cpus().len(),
            memory_used,
            memory_total,
            memory_percent,
            process_count,
            network_rx,
            network_tx,
            disk_usage,
        }
    }

    /// Format bytes to human-readable format
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let _monitor = ResourceMonitor::new();
    }

    #[test]
    fn test_get_stats() {
        let mut monitor = ResourceMonitor::new();
        let stats = monitor.get_stats();
        
        assert!(stats.cpu_count > 0);
        assert!(stats.memory_total > 0);
        assert!(stats.cpu_usage >= 0.0 && stats.cpu_usage <= 100.0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(ResourceMonitor::format_bytes(0), "0.00 B");
        assert_eq!(ResourceMonitor::format_bytes(1024), "1.00 KB");
        assert_eq!(ResourceMonitor::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(ResourceMonitor::format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
