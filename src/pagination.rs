use serde::{Deserialize, Serialize};

const LIMIT: u64 = 100;
const OFFSET: u64 = 0;

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub count: usize,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Deserialize)]
pub struct RequestQuery {
    pub from: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Pagination {
    pub fn build_from_request_query(query: RequestQuery) -> PaginationBuilder {
        let limit = query
            .limit
            .map(|t| std::cmp::min(t, LIMIT))
            .unwrap_or(LIMIT);
        let offset = query.offset.unwrap_or(OFFSET);
        PaginationBuilder {
            count: None,
            limit,
            offset,
        }
    }
}

pub struct PaginationBuilder {
    pub count: Option<usize>,
    pub limit: u64,
    pub offset: u64,
}

impl Default for PaginationBuilder {
    fn default() -> Self {
        Self {
            count: None,
            limit: LIMIT,
            offset: OFFSET,
        }
    }
}

impl PaginationBuilder {
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn build(self) -> Pagination {
        Pagination {
            count: self.count.expect("Pagination count must to be set"),
            limit: self.limit,
            offset: self.offset,
        }
    }
}
