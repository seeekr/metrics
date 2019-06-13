//! Records metrics in the Prometheus exposition format.
use hdrhistogram::Histogram;
use metrics_core::{Key, Recorder};
use metrics_util::{parse_quantiles, Quantile};
use std::collections::HashMap;
use std::time::SystemTime;

/// Records metrics in the Prometheus exposition format.
pub struct PrometheusRecorder {
    quantiles: Vec<Quantile>,
    histograms: HashMap<String, (u64, Histogram<u64>)>,
    output: String,
}

impl PrometheusRecorder {
    /// Creates a new [`PrometheusRecorder`] with a default set of quantiles.
    ///
    /// Configures the recorder with these default quantiles: 0.0, 0.5, 0.9, 0.95, 0.99, 0.999, and
    /// 1.0.  If you want to customize the quantiles used, you can call
    ///   [`PrometheusRecorder::with_quantiles`].
    ///
    /// The configured quantiles are used when rendering any histograms.
    pub fn new() -> Self {
        Self::with_quantiles(&[0.0, 0.5, 0.9, 0.95, 0.99, 0.999, 1.0])
    }

    /// Creates a new [`PrometheusRecorder`] with the given set of quantiles.
    ///
    /// The configured quantiles are used when rendering any histograms.
    pub fn with_quantiles(quantiles: &[f64]) -> Self {
        let actual_quantiles = parse_quantiles(quantiles);
        Self {
            quantiles: actual_quantiles,
            histograms: HashMap::new(),
            output: get_prom_expo_header(),
        }
    }
}

impl Recorder for PrometheusRecorder {
    fn record_counter<K: Into<Key>>(&mut self, key: K, value: u64) {
        let label = key.into().as_ref().replace('.', "_");
        self.output.push_str("\n# TYPE ");
        self.output.push_str(label.as_str());
        self.output.push_str(" counter\n");
        self.output.push_str(label.as_str());
        self.output.push_str(" ");
        self.output.push_str(value.to_string().as_str());
        self.output.push_str("\n");
    }

    fn record_gauge<K: Into<Key>>(&mut self, key: K, value: i64) {
        let label = key.into().as_ref().replace('.', "_");
        self.output.push_str("\n# TYPE ");
        self.output.push_str(label.as_str());
        self.output.push_str(" gauge\n");
        self.output.push_str(label.as_str());
        self.output.push_str(" ");
        self.output.push_str(value.to_string().as_str());
        self.output.push_str("\n");
    }

    fn record_histogram<K: Into<Key>>(&mut self, key: K, values: &[u64]) {
        let mut parts = self.histograms.entry(key.into().to_string()).or_insert_with(|| {
            (0, Histogram::<u64>::new(3).expect("failed to create histogram"))
        });

        for value in values {
            parts.1.record(*value).expect("failed to record histogram value");
            parts.0 += *value;
        }
    }
}

impl Clone for PrometheusRecorder {
    fn clone(&self) -> Self {
        Self {
            output: get_prom_expo_header(),
            histograms: HashMap::new(),
            quantiles: self.quantiles.clone(),
        }
    }
}

impl Into<String> for PrometheusRecorder {
    fn into(self) -> String {
        let mut output = self.output;
        let histograms = self.histograms;

        for (name, (sum, h)) in histograms {
            output.push_str("\n# TYPE ");
            output.push_str(name.as_str());
            output.push_str(" summary\n");

            for quantile in &self.quantiles {
                let value = h.value_at_quantile(quantile.value());
                output.push_str(name.as_str());
                output.push_str("{quantile=\"");
                output.push_str(quantile.value().to_string().as_str());
                output.push_str("\"} ");
                output.push_str(value.to_string().as_str());
                output.push_str("\n");
            }
            output.push_str(name.as_str());
            output.push_str("_sum ");
            output.push_str(sum.to_string().as_str());
            output.push_str("\n");
            output.push_str(name.as_str());
            output.push_str("_count ");
            output.push_str(h.len().to_string().as_str());
            output.push_str("\n");
        }

        output
    }
}

fn get_prom_expo_header() -> String {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    format!(
        "# metrics snapshot (ts={}) (prometheus exposition format)",
        ts
    )
}
