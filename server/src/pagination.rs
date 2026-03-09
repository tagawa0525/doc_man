use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl PaginationParams {
    pub fn validate(&self) -> Result<(), String> {
        if self.page == 0 {
            return Err("page must be >= 1".to_string());
        }
        if self.per_page == 0 || self.per_page > 100 {
            return Err("per_page must be between 1 and 100".to_string());
        }
        Ok(())
    }

    pub fn offset(&self) -> i64 {
        i64::from((self.page.saturating_sub(1)) * self.per_page)
    }

    pub fn limit(&self) -> i64 {
        i64::from(self.per_page)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: u32, per_page: u32) -> Self {
        Self {
            data,
            meta: PaginationMeta {
                total,
                page,
                per_page,
            },
        }
    }
}
