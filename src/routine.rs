use crate::app_state::AppState;
use crate::model::stock;
use chrono::prelude::*;
use chrono::{Duration, NaiveDate, Utc};
use chrono_tz::Asia::Taipei;
use cron::Schedule;
use std::error::Error;
use std::str::FromStr;
pub async fn routine(app: AppState) -> Result<(), Box<dyn Error>> {
    let daily_exp = "0 30 4 * * Tue,Wed,Thu,Fri,Sat *";

    let schedule = Schedule::from_str(daily_exp)?;

    tokio::spawn(async move {
        loop {
            let next = schedule.upcoming(Taipei).next().unwrap();

            let delay = (next - Utc::now().with_timezone(&Taipei)).num_milliseconds() as u64;

            println!("next: {:?}, delay: {:?}", next, delay);

            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

            update_profile(app.clone()).await;

            update_minute_record(app.clone()).await;

            update_daily_price(app.clone()).await;
        }
    });
    // tokio::time::sleep(tokio::time::Duration::from_millis(delay))

    Ok(())
}

#[derive(Debug, serde::Deserialize, sqlx::FromRow, serde::Serialize)]
struct SymbolName {
    pub symbol: String,
}

async fn update_profile(app: AppState) -> Result<(), Box<dyn Error>> {
    let client = app.client;

    let base_url = std::env::var("WEB_SERVICES_URL").expect("WEB_SERVICES_URL is not set.");
    let apikey = std::env::var("WEB_SERVICES_APIKEY").expect("WEB_SERVICES_APIKEY is not set.");

    let symbol_list = sqlx::query_as::<_, SymbolName>("select symbol from demo_app.stock_profile")
        .fetch_all(&app.db)
        .await?;

    let len = symbol_list.len();

    let mut transaction = app.db.begin().await?;

    for n in 0..len {
        let url = base_url.to_owned()
            + "/api/v3/profile/"
            + symbol_list[n].symbol.as_str()
            + "?apikey="
            + apikey.as_str();

        let req = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .json::<stock::FMPCompanyProfile>()
            .await
            .unwrap();

        sqlx::query(
            "
        UPDATE demo_app.stock_profile 
        SET price = ($1),
        beta = ($2),
        vol_avg = ($3),
        mkt_cap = ($4),
        last_div = ($5),
        change = ($6),
        price_range = ($7),
        full_time_employees = ($8),
        is_actively_trading = ($9)
        WHERE symbol = ($10)
      ",
        )
        .bind(&req[0].price)
        .bind(&req[0].beta)
        .bind(&req[0].vol_avg)
        .bind(&req[0].mkt_cap)
        .bind(&req[0].last_div)
        .bind(&req[0].changes)
        .bind(&req[0].range)
        .bind(&req[0].full_time_employees)
        .bind(&req[0].is_actively_trading)
        .bind(&req[0].symbol)
        .execute(&mut transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

async fn update_minute_record(app: AppState) -> Result<(), Box<dyn Error>> {
    let client = app.client;
    let symbol_list = sqlx::query_as::<_, SymbolName>(
        "
      SELECT symbol 
      FROM demo_app.stock_profile 
      WHERE is_actively_trading = TRUE 
      ORDER BY symbol ASC",
    )
    .fetch_all(&app.db)
    .await?;

    let base_url = std::env::var("WEB_SERVICES_URL").expect("WEB_SERVICES_URL is not set.");

    let apikey = std::env::var("WEB_SERVICES_APIKEY").expect("WEB_SERVICES_APIKEY is not set.");

    let mut transaction = app.db.begin().await?;

    let len = symbol_list.len();

    let local = Local::now();

    let last_intraday = match local.weekday() {
        chrono::Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap()
                - Duration::days(3)
        }
        chrono::Weekday::Sun => {
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

    for i in 0..len {
        println!("index:{i}, symbol:{:?}", &symbol_list[i].symbol);
        let url = base_url.to_owned()
            + "/api/v3/historical-chart/1min/"
            + symbol_list[i].symbol.as_str()
            + "?apikey="
            + apikey.as_str();

        let res: Vec<stock::DailyPrice> = client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::DailyPrice>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.date.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();

                return date >= last_intraday;
            })
            .collect();
        let res_len = res.len();

        for n in 0..res_len {
            let date =
                NaiveDateTime::parse_from_str(res[n].date.as_str(), "%Y-%m-%d %H:%M:%S").unwrap();
            sqlx::query(
                "
          INSERT INTO demo_app.stock_price(time, symbol, open, close, high, low, volume) 
          VALUES ($1, $2, $3, $4, $5, $6, $7)
          ",
            )
            .bind(date)
            .bind(&symbol_list[i].symbol)
            .bind(res[n].open)
            .bind(res[n].close)
            .bind(res[n].high)
            .bind(res[n].low)
            .bind(res[n].volume)
            .execute(&mut transaction)
            .await?;
        }
    }
    transaction.commit().await?;

    Ok(())
}

async fn update_daily_price(app: AppState) -> Result<(), Box<dyn Error>> {
    let client = app.client;
    let symbol_list =
      sqlx::query_as::<_, SymbolName>("SELECT symbol FROM demo_app.stock_profile WHERE is_actively_trading = TRUE ORDER BY symbol ASC")
          .fetch_all(&app.db)
          .await?;

    let base_url = std::env::var("WEB_SERVICES_URL").expect("WEB_SERVICES_URL is not set.");
    let apikey = std::env::var("WEB_SERVICES_APIKEY").expect("WEB_SERVICES_APIKEY is not set.");

    let mut transaction = app.db.begin().await?;

    let len = symbol_list.len();

    let local = Local::now();

    let last_intraday = match local.weekday() {
        chrono::Weekday::Mon => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                - Duration::days(3)
        }
        chrono::Weekday::Sun => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                - Duration::days(2)
        }
        _ => {
            NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                - Duration::days(1)
        }
    };

    for i in 0..len {
        println!("index:{i}, symbol: {:?}", symbol_list[i].symbol);
        let ema5 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=5&type=ema&apikey="
            + apikey.as_str();

        let ema20 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=20&type=ema&apikey="
            + apikey.as_str();

        let ema60 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=60&type=ema&apikey="
            + apikey.as_str();

        let sma5 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=5&type=sma&apikey="
            + apikey.as_str();

        let sma20 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=20&type=sma&apikey="
            + apikey.as_str();

        let sma60 = base_url.to_owned()
            + "/api/v3/technical_indicator/daily/"
            + symbol_list[i].symbol.as_str()
            + "?period=60&type=sma&apikey="
            + apikey.as_str();

        let result_ema5: Vec<stock::EmaIndicator> = client
            .get(ema5)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::EmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let result_ema20: Vec<stock::EmaIndicator> = client
            .get(ema20)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::EmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let result_ema60: Vec<stock::EmaIndicator> = client
            .get(ema60)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::EmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let result_sma5: Vec<stock::SmaIndicator> = client
            .get(sma5)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::SmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let result_sma20: Vec<stock::SmaIndicator> = client
            .get(sma20)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::SmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let result_sma60: Vec<stock::SmaIndicator> = client
            .get(sma60)
            .send()
            .await
            .unwrap()
            .json::<Vec<stock::SmaIndicator>>()
            .await
            .unwrap()
            .into_iter()
            .filter(|t| {
                let date =
                    NaiveDateTime::parse_from_str(t.daily.date.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();

                return date > last_intraday;
            })
            .collect();

        let len = result_ema5.len();

        for s in 0..len {
            let date = NaiveDateTime::parse_from_str(
                result_ema5[s].daily.date.as_str(),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap();
            sqlx::query("INSERT INTO demo_app.stock_price(time, symbol, open, close, high, low, ema, sma, volume)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
          .bind(&date)
          .bind(&symbol_list[i].symbol)
          .bind(&result_ema5[s].daily.open)
          .bind(&result_ema5[s].daily.close)
          .bind(&result_ema5[s].daily.high)
          .bind(&result_ema5[s].daily.low)
          .bind([result_ema5[s].ema,result_ema20[s].ema, result_ema60[s].ema])
          .bind([result_sma5[s].sma,result_sma20[s].sma, result_sma60[s].sma])
          .bind(&result_ema5[s].daily.volume).execute(&mut transaction).await?;
        }
    }

    transaction.commit().await?;

    Ok(())
}
