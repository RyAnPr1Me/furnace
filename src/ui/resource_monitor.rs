use sysinfo::{System, Disks};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// System resource monitor for displaying resource usage (optimized with caching)
pub struct ResourceMonitor {
    system: Arc<Mutex<System>>,
    last_update: Instant,
    update_interval: Duration,
    // Cached stats to avoid recomputing when not needed
    cached_stats: Option<ResourceStats>,
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
    /// Create a new resource monitor (optimized initialization)
    pub fn new() -> Self {
        let system = System::new(); // Only initialize what's needed initially
        
        Self {
            system: Arc::new(Mutex::new(system)),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500), // Update every 500ms
            cached_stats: None,
        }
    }

    /// Get current resource statistics (with caching)
    pub fn get_stats(&mut self) -> ResourceStats {
        // Return cached stats if update interval hasn't elapsed
        if self.last_update.elapsed() < self.update_interval {
            if let Some(ref stats) = self.cached_stats {
                return stats.clone();
            }
        }
        
        // Need to update - only refresh what's necessary
        if let Ok(mut system) = self.system.lock() {
            system.refresh_cpu();
            system.refresh_memory();
            // Skip processes refresh if not needed for display
            system.refresh_processes();
        }
        self.last_update = Instant::now();

        let system = self.system.lock().unwrap();

        // CPU usage (average across all cores) - optimized calculation
        let cpus = system.cpus();
        let cpu_count = cpus.len().max(1);
        let cpu_usage = cpus.iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / cpu_count as f32;
        
        // Memory usage
        let memory_used = system.used_memory();
        let memory_total = system.total_memory();
        let memory_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Process count (lightweight)
        let process_count = system.processes().len();

        // Network and disk stats - Basic implementation
        // Note: Advanced network/disk monitoring requires platform-specific APIs
        // Current implementation provides basic disk info available cross-platform
        let network_rx = self.get_network_stats().0;
        let network_tx = self.get_network_stats().1;
        let disk_usage = self.get_disk_info(&system);

        let stats = ResourceStats {
            cpu_usage,
            cpu_count,
            memory_used,
            memory_total,
            memory_percent,
            process_count,
            network_rx,
            network_tx,
            disk_usage,
        };
        
        // Cache the stats
        self.cached_stats = Some(stats.clone());
        stats
    }
    
    /// Get basic network statistics (placeholder for cross-platform implementation)
    /// Returns (rx_bytes, tx_bytes)
    fn get_network_stats(&self) -> (u64, u64) {
        // Basic implementation - can be extended with platform-specific code
        // For now, returns 0 to maintain API compatibility
        (0, 0)
    }
    
    /// Get disk usage information
    fn get_disk_info(&self, _system: &System) -> Vec<DiskInfo> {
        // Get disk information using sysinfo's Disks API
        let disks = Disks::new_with_refreshed_list();
        
        disks.iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                let percent = if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                
                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    used,
                    total,
                    percent,
                }
            })
            .collect()
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
