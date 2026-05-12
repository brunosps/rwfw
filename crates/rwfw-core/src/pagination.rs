use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            ((total as f64) / (per_page as f64)).ceil() as u32
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }

    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) as u64) * (self.per_page as u64)
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}
