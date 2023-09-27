use crate::error::Error;
use crate::model::stock;
use crate::response;
use crate::{app_state::AppState, model::stock::TimeseriesDataBuilder};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::prelude::*;
use chrono::{Duration, Weekday};
use reqwest::header::{ACCEPT, USER_AGENT};
use scraper::{Html, Selector};
use serde::{de, Deserialize, Deserializer};
use serde_json::json;
use serde_with::{serde_as, NoneAsEmptyString};
use std::any::type_name;
use std::{fmt, str::FromStr};
use std::{fs, io::BufWriter};
use tracing_subscriber::fmt::MakeWriter;
use uuid::Uuid;
pub fn routes(app: &mut AppState) -> Router {
    Router::new()
        .route("/stock/sectors", get(list_sectors))
        .route("/stock/symbol", get(list_symbols))
        .route("/stock/symbol/:symbol", get(search_symbol))
        .route("/stock/news", get(list_internal_news))
        .route("/stock/news/:id", get(internal_news).post(other_news))
        .route("/stock/:symbol/news", get(symbol_news))
        .route("/stock/ext_news", get(list_external_news))
        .route("/stock/income_statement/:symbol", get(income_statement))
        .route("/stock/balance_sheet/:symbol", get(balance_sheet))
        .route("/stock/cashflow/:symbol", get(cashflow))
        .route("/stock/profile/:symbol", get(profile))
        .route("/stock/price/daily/:symbol", get(symbol_price_daily))
        .route("/stock/price/:symbol", get(symbol_price))
        .route(
            "/v2/stock/price/:symbol",
            get(symbol_price_with_company_name),
        )
        .route(
            "/stock/price/sector",
            get(list_top_five_mkt_stock_price_each_sector).post(top_five_mkt_stock_price),
        )
        .route("/stock/price/gainer", get(most_gainer))
        .route("/stock/price/loser", get(most_loser))
        .with_state(app.clone())
}

#[derive(Debug, serde::Deserialize)]
struct Params {
    from: i32,
    to: i32,
}

// region: --- route /stock/sectors
async fn list_sectors(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let sectors = sqlx::query_as::<_, stock::Sectors>(
        "SELECT sector_id, sector_name from demo_app.stock_sector ORDER BY sector_id ASC",
    )
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new()
        .body(sectors)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}
// endregion: --- route /stock/sectors

// region: --- route /stock/symbol
async fn list_symbols(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let symbols = sqlx::query_as::<_, stock::Symbols>(
        "SELECT symbol, company_name, img 
        FROM demo_app.stock_profile 
        ORDER BY symbol ASC",
    )
    .fetch_all(&app.db)
    .await?;
    let res = response::CustomResponseBuilder::new().body(symbols).build();

    Ok(res)
}

async fn search_symbol(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<Params>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{}", symbol).to_uppercase() + "%";

    let search = sqlx::query_as::<_, stock::SymbolLists>(
        "with symbol_info as (
                select symbol, company_name, img
                from demo_app.stock_profile 
                where symbol like ($1) or company_name like ($2)
                group by symbol, company_name, img
                order by symbol asc
                offset ($3) rows fetch first ($4) row only
                ),
            count_row as (
                select count(*) as total_row from demo_app.stock_profile
                where symbol like ($1) or company_name like ($2)
                )
            select jsonb_build_object('result',jsonb_agg(si.*), 'pagination', 
            jsonb_build_object('total_rows', count_row.total_row, 'row_count', count(si.*))) as symbol_list, 
            count_row.total_row as total
            from symbol_info si, count_row
            group by total
            ",
    )
    .bind(&s)
    .bind(&s)
    .bind(&params.from)
    .bind(&params.to).fetch_one(&app.db).await?;

    let res = response::CustomResponseBuilder::new().body(search).build();
    Ok(res)
}
// endregion: --- route /stock/

// region: --- route /stock/profile/:symbol
async fn profile(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{}", symbol).to_uppercase();

    let profile = sqlx::query_as::<_, stock::StockProfile>(
        "SELECT * FROM demo_app.stock_profile WHERE symbol = ($1)",
    )
    .bind(&s)
    .fetch_optional(&app.db)
    .await?;

    match profile {
        Some(t) => {
            let res = response::CustomResponseBuilder::new().body(t).build();
            Ok(res)
        }
        None => {
            let message = format!("symbol : `{symbol}`");
            Err(Error::NOTFOUND(message))
        }
    }
}
// endregion: --- route /stock/profile/:symbol

// region: --- route /stock/daily/:symbol
async fn symbol_price_daily(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{symbol}").to_uppercase();

    let stock_price = sqlx::query_as::<_, stock::DailyPrice2>(
        "with daily_cte as (SELECT open, close, high, low, volume, ema, sma, bucket as time FROM daily_intraday_record WHERE symbol = ($1) ORDER BY  bucket ASC)
        select jsonb_build_object('data',jsonb_agg(dc.*), 'symbol', ($1)) as result
        from daily_cte dc 
        ",
    )
    .bind(&s)
    .fetch_one(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new()
        .body(stock_price)
        .build();

    Ok(res)
}
// endregion: --- route /stock/daily/:symbol

// region: --- route /stock/price/:symbol
async fn symbol_price(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let local = Local::now();

    let last_intraday_in_us = match local.weekday() {
        Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(3)
        }
        Weekday::Sun => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(2)
        }
        _ => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(1)
        }
    };

    let s = format!("{symbol}").to_uppercase();

    let price = sqlx::query_as::<_, stock::DailyPriceView>(
        "select time, symbol, open, close, high, low, volume
        from demo_app.stock_price
        where symbol = ($1) and time >= ($2) and ema is null
        order by time asc
        ",
    )
    .bind(&s)
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(price).build();

    Ok(res)
}
// endregion: --- route /stock/price/:symbol

// region: --- route /stock/price/gainer
async fn most_gainer(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let gainer = sqlx::query_as::<_, stock::Gainer>(
        "SELECT symbol, company_name, price, change FROM demo_app.stock_profile 
        ORDER BY ROUND(CAST(change/(price-change)*100 as numeric),2) DESC LIMIT 10",
    )
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(gainer).build();

    Ok(res)
}
// endregion: --- route /stock/price/gainer

// region: --- route /stock/price/loser
async fn most_loser(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let gainer = sqlx::query_as::<_, stock::Gainer>(
        "SELECT symbol, company_name, price, change FROM demo_app.stock_profile 
        ORDER BY ROUND(CAST(change/(price-change)*100 as numeric),2) ASC LIMIT 10",
    )
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(gainer).build();

    Ok(res)
}
// endregion: --- route /stock/price/loser

#[derive(Debug, serde::Deserialize)]
struct SectorId {
    sector_id: i32,
}

// region: --- route /stock/price/sector
async fn list_top_five_mkt_stock_price_each_sector(
    State(app): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let local = Local::now();

    let last_intraday_in_us = match local.weekday() {
        Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(3)
        }
        Weekday::Sun => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(2)
        }
        _ => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(1)
        }
    };

    let price = sqlx::query_as::<_, stock::StockPriceEachSector>(
        "with mkt_cap_cte as (
            select sp.symbol, sp.company_name, sp.mkt_cap, sp.sector_id, sect.sector_name,
                rank() over (partition by sp.sector_id order by sp.mkt_cap desc)
            from demo_app.stock_profile sp
            left join demo_app.stock_sector sect on sect.sector_id = sp.sector_id
        ),
        top_stock as (
            select * from mkt_cap_cte where rank <=5
        )
        select mi.bucket, mi.close, ts.symbol, ts.company_name, ts.mkt_cap, ts.sector_name, ts.sector_id,
            count(ts.symbol) over (partition by ts.symbol order by mi.bucket asc, ts.mkt_cap desc)
        from minute_intraday_record mi
        join top_stock ts on mi.symbol = ts.symbol
        where mi.bucket >= ($1)
        group by mi.bucket, mi.close, ts.symbol, ts.company_name, ts.mkt_cap, ts.sector_name, ts.sector_id
        order by ts.sector_id asc;
        ",
    )
    .bind(&last_intraday_in_us)
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(price).build();

    Ok(res)
}

async fn top_five_mkt_stock_price(
    State(app): State<AppState>,
    Json(id): Json<SectorId>,
) -> Result<impl IntoResponse, Error> {
    let local = Local::now();

    let last_intraday_in_us = match local.weekday() {
        Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(3)
        }
        Weekday::Sun => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(2)
        }
        _ => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(1)
        }
    };

    let price = sqlx::query_as::<_, stock::StockPriceGroupBySector>(
        "
    with mkt_cap_cte as (
        select symbol, company_name, mkt_cap, price, change
        from demo_app.stock_profile
        where sector_id = ($1)
        order by mkt_cap desc
        limit 5
    )
    select mi.bucket, mi.close, mcc.symbol, mcc.company_name, mcc.mkt_cap, mcc.price, mcc.change,
        count(mcc.symbol) over (partition by mcc.symbol order by mi.bucket asc, mcc.mkt_cap desc)
    from minute_intraday_record mi
    join mkt_cap_cte mcc on mi.symbol = mcc.symbol
    where bucket >= ($2)
    group by mi.bucket, mi.close, mcc.symbol, mcc.company_name, mcc.mkt_cap, mcc.price, mcc.change
    ",
    )
    .bind(&id.sector_id)
    .bind(&last_intraday_in_us)
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(price).build();

    Ok(res)
}
// endregion: --- route /stock/price/sector

#[derive(Debug, serde::Deserialize)]
struct Limit {
    number: f32,
}

async fn list_internal_news(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let news = sqlx::query_as::<_, stock::News>(
        "SELECT * FROM demo_app.fmp_news ORDER BY published_date desc LIMIT 50",
    )
    .fetch_all(&app.db)
    .await?;
    let res = response::CustomResponseBuilder::new().body(news).build();
    Ok(res)
}

async fn internal_news(
    State(app): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let news = sqlx::query_as::<_, stock::News>("SELECT * FROM demo_app.fmp_news WHERE id = ($1)")
        .bind(&id)
        .fetch_one(&app.db)
        .await?;
    let res = response::CustomResponseBuilder::new().body(news).build();
    Ok(res)
}

async fn other_news(
    State(app): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let news = sqlx::query_as::<_, stock::OtherNews>(
        "
        SELECT  id, img, title, published_date 
        FROM demo_app.fmp_news 
        WHERE published_date < (SELECT published_date FROM demo_app.fmp_news WHERE id = ($1))
        LIMIT 3
     ",
    )
    .bind(&id)
    .fetch_all(&app.db)
    .await?;
    let res = response::CustomResponseBuilder::new().body(news).build();
    Ok(res)
}

async fn symbol_news(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = "%".to_owned() + format!("{}", symbol).to_uppercase().as_str();

    let news = sqlx::query_as::<_, stock::News>(
        "
    SELECT *
    FROM demo_app.fmp_news
    WHERE SPLIT_PART(tickers,':',2) = ($1) 
    ORDER BY published_date desc
    LIMIT 10
    ",
    )
    .bind(&s)
    .fetch_all(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(news).build();

    Ok(res)
}

async fn list_external_news(State(app): State<AppState>) -> Result<impl IntoResponse, Error> {
    let news = sqlx::query_as::<_, stock::StockNews>(
        "SELECT * FROM demo_app.stock_news ORDER BY published_date desc LIMIT 50",
    )
    .fetch_all(&app.db)
    .await?;
    let res = response::CustomResponseBuilder::new().body(news).build();
    Ok(res)
}

async fn income_statement(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{}", symbol).to_uppercase();
    let income = sqlx::query_as::<_, stock::IncomeStatement>(
        "SELECT symbol, earning_calendar as header, income_statement as rawdata 
        FROM demo_app.company_income
        WHERE symbol = ($1)",
    )
    .bind(&s)
    .fetch_one(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(income).build();

    Ok(res)
}
async fn balance_sheet(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{}", symbol).to_uppercase();
    let sheet = sqlx::query_as::<_, stock::BalanceSheet>(
        "SELECT symbol, earning_calendar as header, balance_sheet as rawdata FROM demo_app.company_balance WHERE symbol = ($1)",
    )
    .bind(&s)
    .fetch_one(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(sheet).build();

    Ok(res)
}
async fn cashflow(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let s = format!("{}", symbol).to_uppercase();
    let flow = sqlx::query_as::<_, stock::CashFlow>(
        "SELECT symbol, earning_calendar as header, cashflow as rawdata FROM demo_app.company_cashflow WHERE symbol = ($1)",
    )
    .bind(&s)
    .fetch_one(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(flow).build();

    Ok(res)
}

async fn symbol_price_with_company_name(
    State(app): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let local = Local::now();

    let last_intraday_in_us = match local.weekday() {
        Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(3)
        }
        Weekday::Sun => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(2)
        }
        _ => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(1)
        }
    };

    let s = format!("{symbol}").to_uppercase();
    let price = sqlx::query_as::<_, stock::DailyPriceWithCompany>(
        "with minute_price as (
            select time, open, close, high, low, volume
            from demo_app.stock_price
            where symbol = ($1) and time >= '2023-09-21 09:30:00.000' and ema is null
            order by time asc), profile_cte as (
                select symbol, price, company_name, exchange_short_name, mkt_cap, change, price_range
                from demo_app.stock_profile
                where symbol = ($1)
            )
        select json_build_object('data', jsonb_agg(mp.*), 'stock', pc.*) as result 
        from minute_price as mp, profile_cte as pc
        group by pc.*
        ",
    )
    .bind(&s)
    .fetch_one(&app.db)
    .await?;

    let res = response::CustomResponseBuilder::new().body(price).build();

    Ok(res)
}
