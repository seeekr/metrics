//! Foundational traits for interoperable metrics libraries in Rust.
//!
//! # Common Ground
//! Most libraries, under the hood, are all based around a core set of data types: counters,
//! gauges, and histograms.  While the API surface may differ, the underlying data is the same.
//!
//! # Metric Types
//!
//! ## Counters
//! Counters represent a single value that can only ever be incremented over time, or reset to
//! zero.
//!
//! Counters are useful for tracking things like operations completed, or errors raised, where
//! the value naturally begins at zero when a process or service is started or restarted.
//!
//! ## Gauges
//! Gauges represent a single value that can go up _or_ down over time.
//!
//! Gauges are useful for tracking things like the current number of connected users, or a stock
//! price, or the temperature outside.
//!
//! ## Histograms
//! Histograms measure the distribution of values for a given set of measurements.
//!
//! Histograms are generally used to derive statistics about a particular measurement from an
//! operation or event that happens over and over, such as the duration of a request, or number of
//! rows returned by a particular database query.
//!
//! Histograms allow you to answer questions of these measurements, such as:
//! - "What were the fastest and slowest requests in this window?"
//! - "What is the slowest request we've seen out of 90% of the requests measured? 99%?"
//!
//! Histograms are a convenient way to measure behavior not only at the median, but at the edges of
//! normal operating behavior.
#![deny(missing_docs)]
use futures::future::Future;
use std::borrow::Cow;
use std::fmt;
use std::slice::Iter;
use std::time::Duration;

/// An allocation-optimized string.
///
/// We specify `ScopedString` to attempt to get the best of both worlds: flexibility to provide a
/// static or dynamic (owned) string, while retaining the performance benefits of being able to
/// take ownership of owned strings and borrows of completely static strings.
pub type ScopedString = Cow<'static, str>;

/// A key/value pair used to further describe a metric.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Label(ScopedString, ScopedString);

impl Label {
    /// Creates a `Label` from a key and value.
    pub fn from_parts<K, V>(key: K, value: V) -> Self
    where
        K: Into<ScopedString>,
        V: Into<ScopedString>,
    {
        Label(key.into(), value.into())
    }
}

/// A metric key.
///
/// A key always includes a name, but can optional include multiple labels used to further describe
/// the metric.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Key {
    #[doc(hidden)]
    pub name: ScopedString,
    #[doc(hidden)]
    pub labels: Option<Vec<Label>>,
}

impl Key {
    /// Creates a `Key` from a name.
    pub fn from_name<N>(name: N) -> Self
    where
        N: Into<ScopedString>,
    {
        Key {
            name: name.into(),
            labels: None,
        }
    }

    /// Creates a `Key` from a name and vector of `Label`s.
    pub fn from_name_and_labels<N, L>(name: N, labels: L) -> Self
    where
        N: Into<ScopedString>,
        L: Into<Vec<Label>>,
    {
        Key {
            name: name.into(),
            labels: Some(labels.into()),
        }
    }

    /// Name of this key.
    pub fn name(&self) -> ScopedString {
        self.name.clone()
    }

    /// Labels of this key, if they exist.
    pub fn labels(&self) -> Option<Iter<Label>> {
        self.labels.as_ref().map(|i| i.iter())
    }

    /// Maps the name of this `Key` to a new name.
    pub fn map_name<F>(self, f: F) -> Self
    where
        F: FnOnce(ScopedString) -> ScopedString,
    {
        Key {
            name: f(self.name),
            labels: self.labels,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.labels {
            None => write!(f, "Key({}", self.name),
            Some(labels) => {
                let kv_pairs = labels
                    .iter()
                    .map(|label| format!("{} = {}", label.0, label.1))
                    .collect::<Vec<_>>();
                write!(f, "Key({}, [{}])", self.name, kv_pairs.join(", "))
            }
        }
    }
}

/// A value that can be converted into a `Key`.
pub trait IntoKey {
    /// Consumes this value, turning it into `Key`.
    fn into_key(self) -> Key;
}

impl IntoKey for Key {
    fn into_key(self) -> Key {
        self
    }
}

impl IntoKey for String {
    fn into_key(self) -> Key {
        Key::from_name(self)
    }
}

impl IntoKey for &'static str {
    fn into_key(self) -> Key {
        Key::from_name(self)
    }
}

impl IntoKey for ScopedString {
    fn into_key(self) -> Key {
        Key::from_name(self)
    }
}

impl<K, L> IntoKey for (K, L)
where
    K: Into<ScopedString>,
    L: IntoLabels,
{
    fn into_key(self) -> Key {
        let labels = self.1.into_labels();
        Key::from_name_and_labels(self.0, labels)
    }
}

/// A value that can be converted to `Label`s.
pub trait IntoLabels {
    /// Consumes this value, turning it into a vector of `Label`s.
    fn into_labels(self) -> Vec<Label>;
}

impl<T> IntoLabels for T
where
    T: Iterator<Item = (ScopedString, ScopedString)>,
{
    fn into_labels(self) -> Vec<Label> {
        self.map(|(k, v)| Label::from_parts(k, v))
            .collect::<Vec<_>>()
    }
}

/// Used to do a nanosecond conversion.
///
/// This trait allows us to interchangably accept raw integer time values, ones already in
/// nanoseconds, as well as the more conventional [`Duration`] which is a result of getting the
/// difference between two [`Instant`](std::time::Instant)s.
pub trait AsNanoseconds {
    /// Performs the conversion.
    fn as_nanos(&self) -> u64;
}

impl AsNanoseconds for u64 {
    fn as_nanos(&self) -> u64 {
        *self
    }
}

impl AsNanoseconds for Duration {
    fn as_nanos(&self) -> u64 {
        self.as_nanos() as u64
    }
}

/// A value that records metrics.
pub trait Recorder {
    /// Records a counter.
    ///
    /// From the perspective of an recorder, a counter and gauge are essentially identical, insofar
    /// as they are both a single value tied to a key.  From the perspective of a collector,
    /// counters and gauges usually have slightly different modes of operation.
    ///
    /// For the sake of flexibility on the exporter side, both are provided.
    fn record_counter(&mut self, key: Key, value: u64);

    /// Records a gauge.
    ///
    /// From the perspective of a recorder, a counter and gauge are essentially identical, insofar
    /// as they are both a single value tied to a key.  From the perspective of a collector,
    /// counters and gauges usually have slightly different modes of operation.
    ///
    /// For the sake of flexibility on the exporter side, both are provided.
    fn record_gauge(&mut self, key: Key, value: i64);

    /// Records a histogram.
    ///
    /// Recorders are expected to tally their own histogram views, so this will be called with all
    /// of the underlying observed values, and callers will need to process them accordingly.
    ///
    /// There is no guarantee that this method will not be called multiple times for the same key.
    fn record_histogram(&mut self, key: Key, values: &[u64]);
}

/// A value that holds a point-in-time view of collected metrics.
pub trait Snapshot {
    /// Records the snapshot to the given recorder.
    fn record<R: Recorder>(&self, recorder: &mut R);
}

/// A value that can provide on-demand snapshots.
pub trait SnapshotProvider {
    /// Snapshot given by the provider.
    type Snapshot: Snapshot;
    /// Errors produced during generation.
    type SnapshotError;

    /// Gets a snapshot.
    fn get_snapshot(&self) -> Result<Self::Snapshot, Self::SnapshotError>;
}

/// A value that can provide on-demand snapshots asynchronously.
pub trait AsyncSnapshotProvider {
    /// Snapshot given by the provider.
    type Snapshot: Snapshot;
    /// Errors produced during generation.
    type SnapshotError;
    /// The future response value.
    type SnapshotFuture: Future<Item = Self::Snapshot, Error = Self::SnapshotError>;

    /// Gets a snapshot asynchronously.
    fn get_snapshot_async(&self) -> Self::SnapshotFuture;
}
