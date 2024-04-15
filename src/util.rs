use std::collections::BTreeMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use error_chain::bail;

use crate::errors::Result;

#[must_use]
pub fn build_request(parameters: BTreeMap<String, String>) -> String {
    let mut request = String::new();
    for (key, value) in parameters {
        let param = format!("{key}={value}&");
        request.push_str(param.as_ref());
    }
    request.pop();
    request
}

/// Build a signed request
///
/// # Errors
///
/// Returns an error if the timestamp cannot be generated.
pub fn build_signed_request(
    parameters: BTreeMap<String, String>,
    recv_window: u64,
) -> Result<String> {
    build_signed_request_custom(parameters, recv_window, SystemTime::now())
}

/// Build a signed request with a custom start time
///
/// # Errors
///
/// Returns an error if the timestamp cannot be generated.
pub fn build_signed_request_custom(
    mut parameters: BTreeMap<String, String>,
    recv_window: u64,
    start: SystemTime,
) -> Result<String> {
    if recv_window > 0 {
        parameters.insert("recvWindow".into(), recv_window.to_string());
    }
    if let Ok(timestamp) = get_timestamp(start) {
        parameters.insert("timestamp".into(), timestamp.to_string());
        return Ok(build_request(parameters));
    }
    bail!("Failed to get timestamp")
}

fn get_timestamp(start: SystemTime) -> Result<u64> {
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000)
}
