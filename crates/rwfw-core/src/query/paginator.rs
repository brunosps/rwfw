use super::PaginationMeta;

pub fn calculate_pagination(page: u32, per_page: u32, total_rows: u64) -> PaginationMeta {
    let total_pages = if total_rows == 0 {
        1
    } else {
        ((total_rows as f64) / (per_page as f64)).ceil() as u32
    };

    PaginationMeta {
        page,
        per_page,
        total_pages,
        total_rows,
    }
}

pub fn offset(page: u32, per_page: u32) -> u64 {
    ((page.saturating_sub(1)) as u64) * (per_page as u64)
}
