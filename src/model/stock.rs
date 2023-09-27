use crate::error::Error;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, types::Decimal, Error as SqlxError, FromRow, Row};
use uuid::Uuid;
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Sectors {
    sector_id: i32,
    sector_name: String,
}

#[derive(Debug, Deserialize)]
pub struct FMPStockNews {
    pub symbol: String,
    #[serde(rename = "publishedDate")]
    pub published_date: String,
    pub title: String,
    pub image: Option<String>,
    pub site: String,
    pub text: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StockNews {
    pub id: Uuid,
    pub symbol: String,
    pub published_date: NaiveDateTime,
    pub title: String,
    pub img: String,
    pub website: String,
    pub content: String,
    pub origin_url: String,
}

#[derive(Debug, Deserialize)]
pub struct CompanyProfile {
    pub symbol: String,
    pub price: f32,
    pub beta: f32,
    #[serde(rename = "volAvg")]
    pub vol_avg: Decimal,
    #[serde(rename = "mktCap")]
    pub mkt_cap: Decimal,
    #[serde(rename = "lastDiv")]
    pub last_div: f32,
    pub changes: f32,
    pub range: String,
    #[serde(rename = "companyName")]
    pub company_name: String,
    pub currency: Option<String>,
    pub cik: Option<String>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub exchange: String,
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: String,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub ceo: Option<String>,
    pub sector: String,
    pub country: Option<String>,
    #[serde(rename = "fullTimeEmployees")]
    pub full_time_employees: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "dcfDiff")]
    pub dcf_diff: Option<f32>,
    pub dcf: f32,
    pub image: Option<String>,
    #[serde(rename = "ipoDate")]
    pub ipo_date: String,
    #[serde(rename = "defaultImage")]
    pub default_image: bool,
    #[serde(rename = "isEtf")]
    pub is_etf: bool,
    #[serde(rename = "isActivelyTrading")]
    pub is_actively_trading: bool,
    #[serde(rename = "isAdr")]
    pub is_adr: bool,
    #[serde(rename = "isFund")]
    pub is_fund: bool,
}

pub type FMPCompanyProfile = [CompanyProfile; 1];

#[derive(Debug, Serialize, FromRow)]
pub struct StockProfile {
    pub id: i32,
    pub symbol: String,
    pub price: f32,
    pub beta: f32,
    pub vol_avg: Decimal,
    pub mkt_cap: Decimal,
    pub last_div: f32,
    pub change: f32,
    pub price_range: String,
    pub company_name: String,
    pub currency: Option<String>,
    pub cik: Option<String>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub exchange: String,
    pub exchange_short_name: String,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub company_description: Option<String>,
    pub ceo: Option<String>,
    pub sector_id: i32,
    pub country: Option<String>,
    pub full_time_employees: Option<String>,
    pub phone: Option<String>,
    pub company_address: Option<String>,
    pub city: Option<String>,
    pub in_state: Option<String>,
    pub zip: Option<String>,
    pub dcf_diff: Option<f32>,
    pub dcf: f32,
    pub img: Option<String>,
    pub ipo_date: Option<NaiveDate>,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Gainer {
    pub symbol: String,
    pub company_name: String,
    pub change: f32,
    pub price: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Symbols {
    pub symbol: String,
    pub company_name: String,
    pub img: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SymbolLists {
    pub symbol_list: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyPrice {
    pub date: String,
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub volume: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DailyPrice2 {
    pub result: Value,
}

#[derive(Debug, Deserialize)]
pub struct Histroical {
    #[serde(flatten)]
    pub daily: DailyPrice,
    #[serde(rename = "adjClose")]
    pub adj_close: f64,
    #[serde(rename = "unadjustedVolume")]
    pub unadjusted_volume: i32,
    pub change: f32,
    #[serde(rename = "changePercent")]
    pub change_percent: f32,
    pub vwap: Option<f32>,
    pub label: String,
    #[serde(rename = "changeOverTime")]
    pub change_over_time: f32,
}

#[derive(Debug, Deserialize)]
pub struct HistroicalStockPrice {
    pub symbol: String,
    pub historical: Vec<Histroical>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmaIndicator {
    #[serde(flatten)]
    pub daily: DailyPrice,
    pub ema: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmaIndicator {
    #[serde(flatten)]
    pub daily: DailyPrice,
    pub sma: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockPrice {
    bucket: NaiveDateTime,
    symbol: String,
    open: f32,
    close: f32,
    high: f32,
    low: f32,
    ema: [f32; 3],
    sma: [f32; 3],
    volume: f32,
}

impl<'r> FromRow<'r, PgRow> for StockPrice {
    fn from_row(row: &'r PgRow) -> Result<Self, SqlxError> {
        let bucket = row.try_get("bucket")?;
        let symbol = row.try_get("symbol")?;
        let open = row.try_get("open")?;
        let close = row.try_get("close")?;
        let high = row.try_get("high")?;
        let low = row.try_get("low")?;
        let ema = row.try_get("ema")?;
        let sma = row.try_get("sma")?;
        let volume = row.try_get("volume")?;

        Ok(StockPrice {
            bucket,
            symbol,
            open,
            close,
            high,
            low,
            ema,
            sma,
            volume,
        })
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct DailyPriceView {
    time: NaiveDateTime,
    open: f32,
    close: f32,
    high: f32,
    low: f32,
    volume: f32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DailyPriceWithCompany {
    result: Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StockPriceGroupBySector {
    pub bucket: NaiveDateTime,
    pub close: f32,
    pub company_name: String,
    pub mkt_cap: Decimal,
    pub symbol: String,
    pub count: i64,
    pub price: f32,
    pub change: f32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StockPriceEachSector {
    bucket: NaiveDateTime,
    close: f32,
    symbol: String,
    company_name: String,
    sector_name: String,
    sector_id: i32,
    mkt_cap: Decimal,
    count: i64,
    // rank: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Timeseries {
    bucket: NaiveDateTime,
    close: f32,
}

#[derive(Debug, Serialize)]
pub struct TimeseriesData {
    symbol: String,
    company_name: String,
    price: f32,
    change: f32,
    mkt_cap: Decimal,
    timeseries: Vec<Timeseries>,
}

#[derive(Default, Clone)]
pub struct TimeseriesDataBuilder {
    symbol: Option<String>,
    company_name: Option<String>,
    price: Option<f32>,
    change: Option<f32>,
    mkt_cap: Option<Decimal>,
    timeseries: Vec<Timeseries>,
}

impl TimeseriesDataBuilder {
    pub fn new() -> Self {
        TimeseriesDataBuilder::default()
    }

    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol.insert(symbol.into());
        self
    }

    pub fn company_name(mut self, company_name: impl Into<String>) -> Self {
        self.company_name.insert(company_name.into());
        self
    }

    pub fn price(mut self, price: impl Into<f32>) -> Self {
        self.price.insert(price.into());
        self
    }

    pub fn change(mut self, change: impl Into<f32>) -> Self {
        self.change.insert(change.into());
        self
    }

    pub fn mkt_cap(mut self, mkt_cap: impl Into<Decimal>) -> Self {
        self.mkt_cap.insert(mkt_cap.into());
        self
    }

    pub fn series_data_init(mut self) -> Self {
        let list: Vec<Timeseries> = vec![];
        self.timeseries = list;
        self
    }

    pub fn series_data(mut self, bucket: impl Into<NaiveDateTime>, val: impl Into<f32>) -> Self {
        self.timeseries.push(Timeseries {
            bucket: bucket.into(),
            close: val.into(),
        });
        self
    }
    pub fn build(self) -> Result<TimeseriesData, Error> {
        let Some(symbol) = self.symbol else {
            return Err(Error::CUSTOMERROR("symbol can not be empty.".to_string()));
        };

        let Some(company_name) = self.company_name else {
            return Err(Error::CUSTOMERROR("symbol can not be empty.".to_string()));
        };

        let Some(price) = self.price else {
            return Err(Error::CUSTOMERROR("symbol can not be empty.".to_string()));
        };

        let Some(change) = self.change else {
            return Err(Error::CUSTOMERROR("change can not be empty.".to_string()));
        };

        let Some(mkt_cap) = self.mkt_cap else {
            return Err(Error::CUSTOMERROR("symbol can not be empty.".to_string()));
        };

        Ok(TimeseriesData {
            symbol,
            company_name,
            price,
            change,
            mkt_cap,
            timeseries: self.timeseries,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct News {
    pub id: Uuid,
    pub title: String,
    pub published_date: NaiveDateTime,
    pub content: Value,
    pub tickers: String,
    pub img: String,
    pub link: String,
    pub author: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct OtherNews {
    pub id: Uuid,
    pub title: String,
    pub published_date: NaiveDateTime,
    pub img: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IncomeStatement {
    pub symbol: String,
    pub header: Vec<String>,
    pub rawdata: Value,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BalanceSheet {
    pub symbol: String,
    pub header: Vec<String>,
    pub rawdata: Value,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CashFlow {
    pub symbol: String,
    pub header: Vec<String>,
    pub rawdata: Value,
}
