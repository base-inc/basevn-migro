//! Source configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source configuration enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceConfig {
    Csv {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        csv: Option<CsvOptions>,
    },
    Excel {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        excel: Option<ExcelOptions>,
    },
    RestApi {
        #[serde(flatten)]
        rest_api: Box<RestApiOptions>,
    },
}

/// CSV-specific options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvOptions {
    /// Delimiter character.
    #[serde(default = "default_csv_delimiter")]
    pub delimiter: String,

    /// Quote character.
    #[serde(default = "default_csv_quote_char")]
    pub quote_char: String,

    /// Has header row.
    #[serde(default = "default_csv_has_header")]
    pub has_header: bool,

    /// Skip N rows before header.
    #[serde(default)]
    pub skip_rows: usize,

    /// Encoding.
    #[serde(default = "default_csv_encoding")]
    pub encoding: String,
}

fn default_csv_delimiter() -> String {
    ",".to_string()
}

fn default_csv_quote_char() -> String {
    "\"".to_string()
}

fn default_csv_has_header() -> bool {
    true
}

fn default_csv_encoding() -> String {
    "utf-8".to_string()
}

/// Excel-specific options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelOptions {
    /// Sheet name or index.
    #[serde(default = "default_excel_sheet")]
    pub sheet: String,

    /// Header row number (1-based).
    #[serde(default = "default_excel_header_row")]
    pub header_row: usize,

    /// Skip N rows after header.
    #[serde(default)]
    pub skip_rows: usize,

    /// Optional cell range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

fn default_excel_sheet() -> String {
    "Sheet1".to_string()
}

fn default_excel_header_row() -> usize {
    1
}

/// REST API source options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestApiOptions {
    /// API URL.
    pub url: String,

    /// HTTP method.
    #[serde(default = "default_rest_method")]
    pub method: String,

    /// Request headers.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Query parameters.
    #[serde(default)]
    pub query_params: HashMap<String, String>,

    /// Request body (for POST).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Pagination configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationConfig>,
}

fn default_rest_method() -> String {
    "GET".to_string()
}

/// Pagination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Pagination type.
    #[serde(rename = "type")]
    pub page_type: PaginationType,

    /// Limit parameter name.
    pub limit_param: String,

    /// Offset parameter name (for offset-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_param: Option<String>,

    /// Cursor parameter name (for cursor-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor_param: Option<String>,

    /// Page parameter name (for page-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_param: Option<String>,

    /// Page size.
    #[serde(default = "default_pagination_limit")]
    pub limit: usize,

    /// JSONPath to records array in response.
    pub response_path: String,

    /// JSONPath to next cursor (for cursor-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor_path: Option<String>,
}

fn default_pagination_limit() -> usize {
    100
}

/// Pagination type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaginationType {
    Offset,
    Cursor,
    Page,
}
