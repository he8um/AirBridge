use std::collections::HashMap;

/// Maximum records per page allowed by the Airtable API.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Default page size used when none is specified.
pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Opaque cursor string returned by Airtable when more pages exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationOffset(pub String);

impl PaginationOffset {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Page size for a list records request.
///
/// Values above `MAX_PAGE_SIZE` are clamped to the maximum.
/// Values below 1 are set to 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize(u32);

impl PageSize {
    pub fn new(requested: u32) -> Self {
        PageSize(requested.clamp(1, MAX_PAGE_SIZE))
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for PageSize {
    fn default() -> Self {
        PageSize(DEFAULT_PAGE_SIZE)
    }
}

/// Options controlling a paginated list records request.
#[derive(Debug, Clone, Default)]
pub struct ListRecordsOptions {
    pub page_size: Option<PageSize>,
    pub offset: Option<PaginationOffset>,
    pub fields: Option<Vec<String>>,
    pub sort_field: Option<String>,
    pub sort_direction: Option<SortDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

/// Converts `ListRecordsOptions` into URL query parameters.
pub fn build_list_query_params(opts: &ListRecordsOptions) -> HashMap<String, String> {
    let mut params = HashMap::new();

    let size = opts.page_size.unwrap_or_default().value();
    params.insert("pageSize".to_string(), size.to_string());

    if let Some(ref offset) = opts.offset {
        params.insert("offset".to_string(), offset.0.clone());
    }

    if let Some(ref fields) = opts.fields {
        for (i, f) in fields.iter().enumerate() {
            params.insert(format!("fields[{i}]"), f.clone());
        }
    }

    if let Some(ref field) = opts.sort_field {
        params.insert("sort[0][field]".to_string(), field.clone());
        let dir = opts.sort_direction.unwrap_or(SortDirection::Asc).as_str();
        params.insert("sort[0][direction]".to_string(), dir.to_string());
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_size_clamps_above_max() {
        assert_eq!(PageSize::new(200).value(), 100);
    }

    #[test]
    fn page_size_clamps_at_max() {
        assert_eq!(PageSize::new(100).value(), 100);
    }

    #[test]
    fn page_size_allows_values_below_max() {
        assert_eq!(PageSize::new(50).value(), 50);
    }

    #[test]
    fn page_size_clamps_zero_to_one() {
        assert_eq!(PageSize::new(0).value(), 1);
    }

    #[test]
    fn default_page_size_is_max() {
        assert_eq!(PageSize::default().value(), DEFAULT_PAGE_SIZE);
        assert_eq!(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE);
    }

    #[test]
    fn build_query_params_includes_page_size() {
        let opts = ListRecordsOptions {
            page_size: Some(PageSize::new(50)),
            ..Default::default()
        };
        let params = build_list_query_params(&opts);
        assert_eq!(params.get("pageSize").map(|s| s.as_str()), Some("50"));
    }

    #[test]
    fn build_query_params_includes_offset() {
        let opts = ListRecordsOptions {
            offset: Some(PaginationOffset("cursor_abc123".to_string())),
            ..Default::default()
        };
        let params = build_list_query_params(&opts);
        assert_eq!(
            params.get("offset").map(|s| s.as_str()),
            Some("cursor_abc123")
        );
    }

    #[test]
    fn build_query_params_omits_offset_when_none() {
        let opts = ListRecordsOptions::default();
        let params = build_list_query_params(&opts);
        assert!(!params.contains_key("offset"));
    }

    #[test]
    fn build_query_params_includes_sort() {
        let opts = ListRecordsOptions {
            sort_field: Some("Name".to_string()),
            sort_direction: Some(SortDirection::Desc),
            ..Default::default()
        };
        let params = build_list_query_params(&opts);
        assert_eq!(
            params.get("sort[0][field]").map(|s| s.as_str()),
            Some("Name")
        );
        assert_eq!(
            params.get("sort[0][direction]").map(|s| s.as_str()),
            Some("desc")
        );
    }
}
