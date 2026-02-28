//! GC Metrics - Export Metrics
//!
//! Module for exporting metrics to monitoring systems
//! (Prometheus, Grafana, etc.)

use crate::error::{FgcError, Result};
use indexmap::IndexMap;
use std::sync::Mutex;

/// GcMetrics - metrics exporter
///
/// Export GC metrics in various formats.
pub struct GcMetrics {
    /// Metrics data
    metrics: Mutex<IndexMap<String, MetricValue>>,
}

impl GcMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(IndexMap::new()),
        }
    }

    /// Add metric
    pub fn add(&self, name: String, value: MetricValue) -> Result<()> {
        self.metrics
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("metrics mutex poisoned: {}", e)))?
            .insert(name, value);
        Ok(())
    }

    /// Get metric
    pub fn get(&self, name: &str) -> Result<Option<MetricValue>> {
        Ok(self
            .metrics
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("metrics mutex poisoned: {}", e)))?
            .get(name)
            .copied())
    }

    /// Export to Prometheus format
    pub fn to_prometheus(&self) -> Result<String> {
        let metrics = self
            .metrics
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("metrics mutex poisoned: {}", e)))?;
        let mut output = String::new();

        for (name, value) in metrics.iter() {
            output.push_str(&format!("{} {}\n", name, value.as_f64()));
        }

        Ok(output)
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String> {
        let metrics = self
            .metrics
            .lock()
            .map_err(|e| FgcError::LockPoisoned(format!("metrics mutex poisoned: {}", e)))?;
        let mut pairs = Vec::new();

        for (name, value) in metrics.iter() {
            pairs.push(format!("\"{}\": {}", name, value.as_f64()));
        }

        Ok(format!("{{{}}}", pairs.join(",")))
    }
}

impl Default for GcMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric value
#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(u64),
}

impl MetricValue {
    pub fn as_f64(&self) -> f64 {
        match self {
            MetricValue::Counter(v) => *v as f64,
            MetricValue::Gauge(v) => *v,
            MetricValue::Histogram(v) => *v as f64,
        }
    }
}
