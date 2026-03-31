//! Client for fetching basis set data from the BSE REST API.
//!
//! This module provides functionality to fetch basis set data from the
//! Basis Set Exchange REST API at `https://www.basissetexchange.org`.
//!
//! The API URL can be overridden via the `BSE_API_URL` environment variable.

use crate::prelude::*;

/// Default URL for the BSE REST API.
pub static DEFAULT_API_URL: &str = "https://www.basissetexchange.org";

/// Get the API URL from environment variable or use default.
pub fn get_api_url() -> String {
    std::env::var("BSE_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string())
}

/// Fetch metadata for all basis sets from the remote API.
///
/// Returns a HashMap of basis set names to their metadata.
pub fn get_metadata_remote(api_url: Option<&str>) -> Result<HashMap<String, BseRootMetadata>, BseError> {
    let base_url = api_url.map(|s| s.to_string()).unwrap_or_else(get_api_url);
    let url = format!("{}/api/metadata", base_url);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch metadata from API. Status: {}. Error: {}",
            status,
            error_body
        )?;
    }

    let metadata: HashMap<String, BseRootMetadata> = response
        .into_body()
        .read_json()
        .map_err(|e| BseError::SerdeJsonError(format!("Failed to parse JSON response: {}", e)))?;

    Ok(metadata)
}

/// Fetch a basis set as JSON from the remote API.
///
/// This fetches the basis set data and deserializes it into a `BseBasis`
/// struct. The REST API handles element selection and basic manipulations
/// (uncontract, optimize, etc).
///
/// Note: The REST API does NOT support `augment_diffuse`, `augment_steep`,
/// `get_aux`, or `remove_free_primitives`. These must be applied locally after
/// fetching.
pub fn get_basis_remote(name: &str, args: &BseGetBasisArgs) -> Result<BseBasis, BseError> {
    let base_url = args.api_url.clone().unwrap_or_else(get_api_url);
    let url = build_basis_url(&base_url, name, "json", args);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch basis set '{}' from API. Status: {}. Error: {}",
            name,
            status,
            error_body
        )?;
    }

    let basis: BseBasis = response
        .into_body()
        .read_json()
        .map_err(|e| BseError::SerdeJsonError(format!("Failed to parse JSON response: {}", e)))?;

    Ok(basis)
}

/// Fetch a formatted basis set from the remote API.
///
/// This fetches the basis set in a specific format (nwchem, gaussian94, etc.)
/// as a formatted string, ready to use in quantum chemistry software.
pub fn get_formatted_basis_remote(name: &str, fmt: &str, args: &BseGetBasisArgs) -> Result<String, BseError> {
    let base_url = args.api_url.clone().unwrap_or_else(get_api_url);
    let url = build_basis_url(&base_url, name, fmt, args);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch basis set '{}' in format '{}' from API. Status: {}. Error: {}",
            name,
            fmt,
            status,
            error_body
        )?;
    }

    let formatted = response
        .into_body()
        .read_to_string()
        .map_err(|e| BseError::IOError(format!("Failed to read response body: {}", e)))?;

    Ok(formatted)
}

/// Build the URL for the basis set API endpoint with query parameters.
fn build_basis_url(base_url: &str, name: &str, fmt: &str, args: &BseGetBasisArgs) -> String {
    let mut url = format!("{}/api/basis/{}/format/{}", base_url, name, fmt);

    let mut params: Vec<String> = Vec::new();

    if let Some(elements) = &args.elements {
        params.push(format!("elements={}", url_encode(elements)));
    }

    if let Some(version) = &args.version {
        params.push(format!("version={}", url_encode(version)));
    }

    if args.uncontract_general {
        params.push("uncontract_general=true".to_string());
    }

    if args.uncontract_segmented {
        params.push("uncontract_segmented=true".to_string());
    }

    if args.uncontract_spdf {
        params.push("uncontract_spdf=true".to_string());
    }

    if args.optimize_general {
        params.push("optimize_general=true".to_string());
    }

    if args.make_general {
        params.push("make_general=true".to_string());
    }

    if !args.header {
        params.push("header=false".to_string());
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    url
}

/// Simple URL encoding for query parameter values.
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '!' => "%21".to_string(),
            '#' => "%23".to_string(),
            '$' => "%24".to_string(),
            '%' => "%25".to_string(),
            '&' => "%26".to_string(),
            '\'' => "%27".to_string(),
            '(' => "%28".to_string(),
            ')' => "%29".to_string(),
            '*' => "%2A".to_string(),
            '+' => "%2B".to_string(),
            ',' => "%2C".to_string(),
            '/' => "%2F".to_string(),
            ':' => "%3A".to_string(),
            ';' => "%3B".to_string(),
            '=' => "%3D".to_string(),
            '?' => "%3F".to_string(),
            '@' => "%40".to_string(),
            '[' => "%5B".to_string(),
            ']' => "%5D".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

/// Fetch available formats from the remote API.
pub fn get_formats_remote(api_url: Option<&str>) -> Result<HashMap<String, String>, BseError> {
    let base_url = api_url.map(|s| s.to_string()).unwrap_or_else(get_api_url);
    let url = format!("{}/api/formats", base_url);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch formats from API. Status: {}. Error: {}",
            status,
            error_body
        )?;
    }

    let formats: HashMap<String, String> = response
        .into_body()
        .read_json()
        .map_err(|e| BseError::SerdeJsonError(format!("Failed to parse JSON response: {}", e)))?;

    Ok(formats)
}

/// Fetch reference formats from the remote API.
pub fn get_reference_formats_remote(api_url: Option<&str>) -> Result<HashMap<String, String>, BseError> {
    let base_url = api_url.map(|s| s.to_string()).unwrap_or_else(get_api_url);
    let url = format!("{}/api/reference_formats", base_url);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch reference formats from API. Status: {}. Error: {}",
            status,
            error_body
        )?;
    }

    let formats: HashMap<String, String> = response
        .into_body()
        .read_json()
        .map_err(|e| BseError::SerdeJsonError(format!("Failed to parse JSON response: {}", e)))?;

    Ok(formats)
}

/// Fetch notes for a basis set from the remote API.
pub fn get_basis_notes_remote(name: &str, api_url: Option<&str>) -> Result<String, BseError> {
    let base_url = api_url.map(|s| s.to_string()).unwrap_or_else(get_api_url);
    let url = format!("{}/api/notes/{}", base_url, name);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch notes for basis set '{}' from API. Status: {}. Error: {}",
            name,
            status,
            error_body
        )?;
    }

    let notes = response
        .into_body()
        .read_to_string()
        .map_err(|e| BseError::IOError(format!("Failed to read response body: {}", e)))?;

    Ok(notes)
}

/// Fetch notes for a basis set family from the remote API.
pub fn get_family_notes_remote(family: &str, api_url: Option<&str>) -> Result<String, BseError> {
    let base_url = api_url.map(|s| s.to_string()).unwrap_or_else(get_api_url);
    let url = format!("{}/api/family_notes/{}", base_url, family);

    let response = ureq::get(&url)
        .header("User-Agent", "bse-rs (Basis Set Exchange in Rust)")
        .header("From", "bse-rs@example.com")
        .call()
        .map_err(|e| BseError::IOError(format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        let error_body = response.into_body().read_to_string().unwrap_or_else(|_| "Unknown error".to_string());
        return bse_raise!(
            DataNotFound,
            "Failed to fetch notes for family '{}' from API. Status: {}. Error: {}",
            family,
            status,
            error_body
        )?;
    }

    let notes = response
        .into_body()
        .read_to_string()
        .map_err(|e| BseError::IOError(format!("Failed to read response body: {}", e)))?;

    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metadata_remote() {
        let metadata = get_metadata_remote(None).unwrap();
        assert!(!metadata.is_empty());
        println!("Found {} basis sets", metadata.len());
    }

    #[test]
    fn test_get_basis_remote() {
        let args = BseGetBasisArgsBuilder::default().elements("H, C".to_string()).build().unwrap();
        let basis = get_basis_remote("sto-3g", &args).unwrap();
        assert_eq!(basis.name, "STO-3G");
        println!("Basis set: {:?}", basis.name);
    }

    #[test]
    fn test_get_formatted_basis_remote() {
        let args = BseGetBasisArgsBuilder::default().elements("H".to_string()).header(false).build().unwrap();
        let formatted = get_formatted_basis_remote("sto-3g", "nwchem", &args).unwrap();
        assert!(formatted.contains("H"));
        println!("Formatted output:\n{}", formatted);
    }

    #[test]
    fn test_get_formats_remote() {
        let formats = get_formats_remote(None).unwrap();
        assert!(!formats.is_empty());
        println!("Available formats: {:?}", formats.keys().collect::<Vec<_>>());
    }

    #[test]
    fn test_get_basis_notes_remote() {
        let notes = get_basis_notes_remote("6-31g", None).unwrap();
        assert!(!notes.is_empty());
        println!("Notes for 6-31g:\n{}", notes);
    }
}
