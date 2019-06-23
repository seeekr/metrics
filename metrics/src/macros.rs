/// Records a counter.
///
/// Functionally equivalent to calling [`Recorder::record_counter`].
///
/// ### Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate metrics;
/// fn do_thing() {
///     let count: u64 = 42;
///     counter!("do_thing", count);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! counter {
    ($name:expr, $value:expr, $($k:expr => $v:expr),+) => {
        let mut labels = Vec::new();
        $( labels.push(metrics_core::Label::from_parts($k, $v)); )+
        let key = metrics_core::Key::from_name_and_labels($name, labels);
        $crate::__private_api_record_count(key, $value);
    };
    ($name:expr, $value:expr) => {
        $crate::__private_api_record_count(metrics_core::Key::from_name($name), $value);
    };
}

/// Records a gauge.
///
/// Functionally equivalent to calling [`Recorder::record_gauge`].
///
/// ### Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate metrics;
/// fn update_current_value() {
///     let value: i64 = -131;
///     gauge!("current_value", value);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! gauge {
    ($name:expr, $value:expr, $($k:expr => $v:expr),+) => {
        let mut labels = Vec::new();
        $( labels.push(metrics_core::Label::from_parts($k, $v)); )+
        let key = metrics_core::Key::from_name_and_labels($name, labels);
        $crate::__private_api_record_gauge(key, $value);
    };
    ($name:tt, $value:expr) => {
        $crate::__private_api_record_gauge(metrics_core::Key::from_name($name), $value);
    };
}

/// Records a timing.
///
/// Functionally equivalent to calling [`Recorder::record_histogram`].
///
/// ### Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate metrics;
/// # use std::time::Instant;
/// # fn process() {}
/// fn handle_request() {
///     let start = Instant::now();
///     process();
///     let end = Instant::now();
///
///     // We can pass instances of `Instant` directly:
///     timing!("performance.request_processed", start, end);
///
///     // Or we can pass just the delta:
///     let delta = end - start;
///     timing!("performance.request_processed", delta);
///
///     // And we can even pass unsigned values, both for the start/end notation:
///     let start: u64 = 100;
///     let end: u64 = 200;
///     timing!("performance.request_processed", start, end);
///
///     // And the delta notation:
///     let delta: u64 = end - start;
///     timing!("performance.request_processed", delta);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! timing {
    ($name:tt, $value:expr) => {
        $crate::__private_api_record_histogram(metrics_core::Key::from_name($name), $value);
    };
    ($name:tt, $start:expr, $end:expr) => {
        timing!($name, $end - $start)
    };
    ($name:expr, $value:expr, $($k:expr => $v:expr),+) => {
        let mut labels = Vec::new();
        $( labels.push(metrics_core::Label::from_parts($k, $v)); )+
        let key = metrics_core::Key::from_name_and_labels($name, labels);
        $crate::__private_api_record_histogram(key, $value);
    };
    ($name:expr, $start:expr, $end:expr, $($k:expr => $v:expr),+) => {
        let mut labels = Vec::new();
        $( labels.push(metrics_core::Label::from_parts($k, $v)); )+
        let key = metrics_core::Key::from_name_and_labels($name, labels);
        let value = $end - $start;
        $crate::__private_api_record_histogram(key, value);
    };
}

/// Records a value.
///
/// Functionally equivalent to calling [`Recorder::record_histogram`].
///
/// ### Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate metrics;
/// # use std::time::Instant;
/// # fn process() -> u64 { 42 }
/// fn handle_request() {
///     let rows_read = process();
///     value!("client.process_num_rows", rows_read);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! value {
    ($name:expr, $value:expr, $($k:expr => $v:expr),+) => {
        let mut labels = Vec::new();
        $( labels.push(metrics_core::Label::from_parts($k, $v)); )+
        let key = metrics_core::Key::from_name_and_labels($name, labels);
        $crate::__private_api_record_histogram(key, $value);
    };
    ($name:tt, $value:expr) => {
        $crate::__private_api_record_histogram(metrics_core::Key::from_name($name), $value);
    };
}
