//! GC Logging and Tracing
//!
//! Comprehensive logging for GC operations, useful for:
//! - Performance analysis
//! - Debugging
//! - Production monitoring
//!
//! Log Levels:
//! - ERROR: GC failures
//! - WARN: Unusual conditions
//! - INFO: GC cycles, phases
//! - DEBUG: Detailed operations
//! - TRACE: Per-object operations

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Log level for GC operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

/// GC event types
#[derive(Debug, Clone)]
pub enum GcEvent {
    /// GC cycle started
    CycleStart {
        generation: String,
        reason: String,
        cycle: u64,
    },

    /// GC phase started
    PhaseStart { phase: String, cycle: u64 },

    /// GC phase completed
    PhaseEnd {
        phase: String,
        duration_ms: f64,
        cycle: u64,
    },

    /// GC cycle completed
    CycleEnd {
        cycle: u64,
        duration_ms: f64,
        reclaimed_bytes: usize,
    },

    /// Heap statistics
    HeapStats {
        used_bytes: usize,
        total_bytes: usize,
        utilization: f64,
    },

    /// Pause time
    Pause { phase: String, duration_us: u64 },

    /// Allocation failure
    AllocationFailure { size: usize, heap_used: usize },

    /// TLAB statistics
    TlabStats {
        active_count: usize,
        total_allocated: usize,
    },

    /// Marking statistics
    MarkStats {
        marked_count: u64,
        scanned_count: u64,
    },

    /// Relocation statistics
    RelocateStats {
        relocated_count: usize,
        bytes_moved: usize,
    },

    /// Reference processing statistics (ZGC-like)
    ReferenceStats {
        weak_cleared: u64,
        soft_cleared: u64,
        phantom_cleared: u64,
        finalizers_processed: u64,
    },

    /// GC thread statistics
    GcThreadStats {
        thread_id: u64,
        work_items_processed: u64,
        cpu_time_ns: u64,
    },

    /// Memory statistics
    MemoryStats {
        committed_bytes: usize,
        used_bytes: usize,
        reclaimed_bytes: usize,
        large_pages_used: bool,
    },

    /// Adaptive tuning event
    TuningEvent {
        parameter: String,
        old_value: f64,
        new_value: f64,
        reason: String,
    },
}

/// GC Logger configuration
#[derive(Debug, Clone)]
pub struct GcLoggerConfig {
    /// Minimum log level
    pub level: LogLevel,

    /// Enable console output
    pub console: bool,

    /// Enable file output
    pub file: Option<String>,

    /// Enable JSON format
    pub json: bool,

    /// Enable timestamps
    pub timestamps: bool,
}

impl Default for GcLoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            console: true,
            file: None,
            json: false,
            timestamps: true,
        }
    }
}

/// GC Logger - centralized logging for GC operations
pub struct GcLogger {
    config: GcLoggerConfig,
    events: Mutex<Vec<(Instant, GcEvent)>>,
    enabled: AtomicBool,
}

impl GcLogger {
    /// Create new GC logger
    pub fn new(config: GcLoggerConfig) -> Self {
        Self {
            config,
            events: Mutex::new(Vec::new()),
            enabled: AtomicBool::new(true),
        }
    }

    /// Enable logging
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable logging
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Log a GC event
    pub fn log(&self, event: GcEvent) {
        if !self.is_enabled() {
            return;
        }

        let event_level = self.event_level(&event);
        if event_level > self.config.level {
            return;
        }

        let timestamp = Instant::now();

        // Store event
        if let Ok(mut events) = self.events.lock() {
            events.push((timestamp, event.clone()));
        }

        // Output to console
        if self.config.console {
            self.output_console(&event);
        }

        // Output to file
        if let Some(ref _path) = self.config.file {
            // TODO: Implement file output
        }
    }

    /// Get log level for event
    fn event_level(&self, event: &GcEvent) -> LogLevel {
        match event {
            GcEvent::AllocationFailure { .. } => LogLevel::Error,
            GcEvent::CycleStart { .. } | GcEvent::CycleEnd { .. } | GcEvent::HeapStats { .. } => {
                LogLevel::Info
            },
            GcEvent::PhaseStart { .. } | GcEvent::PhaseEnd { .. } | GcEvent::Pause { .. } => {
                LogLevel::Debug
            },
            GcEvent::TlabStats { .. }
            | GcEvent::MarkStats { .. }
            | GcEvent::RelocateStats { .. } => LogLevel::Trace,
        }
    }

    /// Output to console
    fn output_console(&self, event: &GcEvent) {
        if self.config.timestamps {
            let now = chrono::Local::now();
            print!("[{}] ", now.format("%Y-%m-%d %H:%M:%S%.3f"));
        }

        if self.config.json {
            self.output_json(event);
        } else {
            self.output_human(event);
        }
    }

    /// Output in human-readable format
    fn output_human(&self, event: &GcEvent) {
        match event {
            GcEvent::CycleStart {
                generation,
                reason,
                cycle,
            } => {
                println!(
                    "[GC] Cycle {} started ({} generation, reason: {})",
                    cycle, generation, reason
                );
            },
            GcEvent::PhaseStart { phase, cycle } => {
                println!("[GC] Cycle {}: {} phase started", cycle, phase);
            },
            GcEvent::PhaseEnd {
                phase,
                duration_ms,
                cycle,
            } => {
                println!(
                    "[GC] Cycle {}: {} phase completed ({:.2}ms)",
                    cycle, phase, duration_ms
                );
            },
            GcEvent::CycleEnd {
                cycle,
                duration_ms,
                reclaimed_bytes,
            } => {
                println!(
                    "[GC] Cycle {} completed ({:.2}ms, reclaimed {} bytes)",
                    cycle, duration_ms, reclaimed_bytes
                );
            },
            GcEvent::HeapStats {
                used_bytes,
                total_bytes,
                utilization,
            } => {
                println!(
                    "[GC] Heap: {}/{} bytes ({:.1}% utilized)",
                    used_bytes,
                    total_bytes,
                    utilization * 100.0
                );
            },
            GcEvent::Pause { phase, duration_us } => {
                println!("[GC] {} pause: {} us", phase, duration_us);
            },
            GcEvent::AllocationFailure { size, heap_used } => {
                eprintln!(
                    "[GC] Allocation failure: {} bytes (heap used: {})",
                    size, heap_used
                );
            },
            GcEvent::TlabStats {
                active_count,
                total_allocated,
            } => {
                println!(
                    "[GC] TLAB: {} active, {} bytes allocated",
                    active_count, total_allocated
                );
            },
            GcEvent::MarkStats {
                marked_count,
                scanned_count,
            } => {
                println!(
                    "[GC] Marked: {} objects, scanned: {} objects",
                    marked_count, scanned_count
                );
            },
            GcEvent::RelocateStats {
                relocated_count,
                bytes_moved,
            } => {
                println!(
                    "[GC] Relocated: {} objects ({} bytes moved)",
                    relocated_count, bytes_moved
                );
            },
            GcEvent::ReferenceStats {
                weak_cleared,
                soft_cleared,
                phantom_cleared,
                finalizers_processed,
            } => {
                println!(
                    "[GC] References: {} weak, {} soft, {} phantom cleared, {} finalizers",
                    weak_cleared, soft_cleared, phantom_cleared, finalizers_processed
                );
            },
            GcEvent::GcThreadStats {
                thread_id,
                work_items_processed,
                cpu_time_ns,
            } => {
                println!(
                    "[GC] Thread {}: {} items, {} ns CPU",
                    thread_id, work_items_processed, cpu_time_ns
                );
            },
            GcEvent::MemoryStats {
                committed_bytes,
                used_bytes,
                reclaimed_bytes,
                large_pages_used,
            } => {
                println!(
                    "[GC] Memory: {} committed, {} used, {} reclaimed, large_pages={}",
                    committed_bytes, used_bytes, reclaimed_bytes, large_pages_used
                );
            },
            GcEvent::TuningEvent {
                parameter,
                old_value,
                new_value,
                reason,
            } => {
                println!(
                    "[GC] Tuning: {} changed from {} to {} ({})",
                    parameter, old_value, new_value, reason
                );
            },
        }
    }

    /// Output in JSON format
    fn output_json(&self, event: &GcEvent) {
        let json = match event {
            GcEvent::CycleStart {
                generation,
                reason,
                cycle,
            } => serde_json::json!({
                "type": "cycle_start",
                "cycle": cycle,
                "generation": generation,
                "reason": reason
            }),
            GcEvent::PhaseStart { phase, cycle } => serde_json::json!({
                "type": "phase_start",
                "cycle": cycle,
                "phase": phase
            }),
            GcEvent::PhaseEnd {
                phase,
                duration_ms,
                cycle,
            } => serde_json::json!({
                "type": "phase_end",
                "cycle": cycle,
                "phase": phase,
                "duration_ms": duration_ms
            }),
            GcEvent::CycleEnd {
                cycle,
                duration_ms,
                reclaimed_bytes,
            } => serde_json::json!({
                "type": "cycle_end",
                "cycle": cycle,
                "duration_ms": duration_ms,
                "reclaimed_bytes": reclaimed_bytes
            }),
            GcEvent::HeapStats {
                used_bytes,
                total_bytes,
                utilization,
            } => serde_json::json!({
                "type": "heap_stats",
                "used_bytes": used_bytes,
                "total_bytes": total_bytes,
                "utilization": utilization
            }),
            GcEvent::Pause { phase, duration_us } => serde_json::json!({
                "type": "pause",
                "phase": phase,
                "duration_us": duration_us
            }),
            GcEvent::AllocationFailure { size, heap_used } => serde_json::json!({
                "type": "allocation_failure",
                "size": size,
                "heap_used": heap_used
            }),
            GcEvent::TlabStats {
                active_count,
                total_allocated,
            } => serde_json::json!({
                "type": "tlab_stats",
                "active_count": active_count,
                "total_allocated": total_allocated
            }),
            GcEvent::MarkStats {
                marked_count,
                scanned_count,
            } => serde_json::json!({
                "type": "mark_stats",
                "marked_count": marked_count,
                "scanned_count": scanned_count
            }),
            GcEvent::RelocateStats {
                relocated_count,
                bytes_moved,
            } => serde_json::json!({
                "type": "relocate_stats",
                "relocated_count": relocated_count,
                "bytes_moved": bytes_moved
            }),
        };

        if let Ok(json_str) = serde_json::to_string(&json) {
            println!("{}", json_str);
        }
    }

    /// Get all events
    pub fn get_events(&self) -> Vec<(Instant, GcEvent)> {
        if let Ok(events) = self.events.lock() {
            events.clone()
        } else {
            Vec::new()
        }
    }

    /// Clear all events
    pub fn clear_events(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        if let Ok(events) = self.events.lock() {
            events.len()
        } else {
            0
        }
    }
}

impl Default for GcLogger {
    fn default() -> Self {
        Self::new(GcLoggerConfig::default())
    }
}

/// Global GC logger
lazy_static::lazy_static! {
    static ref GLOBAL_LOGGER: Mutex<GcLogger> = Mutex::new(GcLogger::default());
}

/// Log a GC event to global logger
pub fn log_event(event: GcEvent) {
    if let Ok(logger) = GLOBAL_LOGGER.lock() {
        logger.log(event);
    }
}

/// Configure global logger
pub fn configure_logger(config: GcLoggerConfig) {
    if let Ok(mut logger) = GLOBAL_LOGGER.lock() {
        *logger = GcLogger::new(config);
    }
}

/// Get global logger event count
pub fn get_event_count() -> usize {
    if let Ok(logger) = GLOBAL_LOGGER.lock() {
        logger.event_count()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_logger_basic() {
        let logger = GcLogger::default();

        logger.log(GcEvent::CycleStart {
            generation: "Young".to_string(),
            reason: "Allocation".to_string(),
            cycle: 1,
        });

        assert_eq!(logger.event_count(), 1);
    }

    #[test]
    fn test_gc_logger_disable() {
        let logger = GcLogger::default();

        logger.disable();
        logger.log(GcEvent::CycleStart {
            generation: "Young".to_string(),
            reason: "Allocation".to_string(),
            cycle: 1,
        });

        assert_eq!(logger.event_count(), 0);
    }

    #[test]
    fn test_global_logger() {
        log_event(GcEvent::CycleStart {
            generation: "Full".to_string(),
            reason: "Explicit".to_string(),
            cycle: 1,
        });

        assert!(get_event_count() > 0);
    }
}
