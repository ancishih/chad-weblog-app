-- Add up migration script here
CREATE SCHEMA demo_app;
CREATE SCHEMA weblog;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS weblog.session_table (
  ss_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  expires TIMESTAMP WITH TIME ZONE NULL,
  session TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS weblog.user (
  usr_id SERIAL PRIMARY KEY,
  github_id INTEGER UNIQUE,
  username VARCHAR UNIQUE NOT NULL,
  password_hash VARCHAR,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS weblog.who_is_login (
  id SERIAL PRIMARY KEY,
  usr_id INTEGER REFERENCES weblog.user (usr_id) NOT NULL,
  ss_id uuid,
  CONSTRAINT usr_login
    FOREIGN KEY(ss_id)
    REFERENCES weblog.session_table(ss_id)
    ON DELETE SET NULL
    ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS demo_app.stock_sector (
  sector_id SERIAL PRIMARY KEY,
  sector_name VARCHAR(50) UNIQUE,
  created_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS demo_app.stock_profile (
  id SERIAL PRIMARY KEY,
  symbol VARCHAR,
  price FLOAT4,
  beta FLOAT4,
  vol_avg NUMERIC,
  mkt_cap NUMERIC,
  last_div FLOAT4,
  change FLOAT4,
  price_range VARCHAR,
  company_name VARCHAR,
  currency VARCHAR,
  cik VARCHAR,
  isin VARCHAR,
  cusip VARCHAR,
  exchange VARCHAR,
  exchange_short_name VARCHAR,
  industry VARCHAR,
  website VARCHAR,
  company_description TEXT,
  ceo VARCHAR,
  sector_id INT,
  country VARCHAR,
  full_time_employees VARCHAR,
  phone VARCHAR,
  company_address VARCHAR,
  city VARCHAR,
  in_state VARCHAR,
  zip VARCHAR,
  dcf_diff FLOAT4,
  dcf FLOAT4,
  img VARCHAR,
  ipo_date DATE,
  default_image BOOLEAN,
  is_etf BOOLEAN,
  is_actively_trading BOOLEAN,
  is_adr BOOLEAN,
  is_fund BOOLEAN,
  UNIQUE (id, symbol, company_name, exchange_short_name),
  CONSTRAINT sector
    FOREIGN KEY(sector_id)
    REFERENCES demo_app.stock_sector(sector_id)
);

CREATE TABLE IF NOT EXISTS demo_app.company (
  symbol VARCHAR UNIQUE PRIMARY KEY,
  company_name VARCHAR
);

CREATE TABLE IF NOT EXISTS demo_app.stock_news (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  symbol VARCHAR,
  published_date TIMESTAMP,
  title VARCHAR,
  img VARCHAR,
  website VARCHAR,
  content TEXT,
  origin_url VARCHAR
);

CREATE TABLE IF NOT EXISTS demo_app.fmp_news(
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  title VARCHAR,
  published_date TIMESTAMP,
  content  jsonb,
  tickers VARCHAR,
  symbol VARCHAR,
  img VARCHAR,
  link VARCHAR,
  author VARCHAR
)

CREATE TABLE IF NOT EXISTS demo_app.stock_price(
  time TIMESTAMP NOT NULL,
  symbol TEXT NOT NULL,
  open FLOAT4 NULL,
  close FLOAT4 NULL,
  high FLOAT4 NULL,
  low FLOAT4 NULL,
  ema FLOAT4 [] NULL,
  sma FLOAT4 [] NULL,
  volume FLOAT4 NULL
);

SELECT create_hypertable('demo_app.stock_price', 'time');
CREATE INDEX ix_symbol_time ON demo_app.stock_price (symbol, time DESC);

INSERT INTO demo_app.stock_sector (sector_name, created_at) VALUES
('Technology',current_timestamp),
('Communication Services',current_timestamp),
('Consumer Cyclical',current_timestamp),
('Financial Services',current_timestamp),
('Healthcare',current_timestamp),
('Energy',current_timestamp),
('Consumer Defensive',current_timestamp),
('Basic Materials',current_timestamp),
('Industrials',current_timestamp),
('Utilities',current_timestamp),
('Real Estate',current_timestamp);


CREATE TABLE IF NOT EXISTS demo_app.indices (
  id SERIAL PRIMARY KEY,
  index_symbol VARCHAR UNIQUE,
  index_name VARCHAR,
  symbol_with_html VARCHAR 
);


INSERT INTO demo_app.indices (index_symbol, index_name, symbol_with_html) 
VALUES ('^DJI', 'Dow Jones Industrial Average', '%5EDJI'), 
('^GSPC', 'S&P 500', '%5EGSPC'), 
('^IXIC', 'NASDAQ Composite', '%5EIXIC'), 
('^VIX', 'CBOE Volatility Index', '%5EVIX');

CREATE TABLE IF NOT EXISTS demo_app.commodities (
  id SERIAL PRIMARY KEY,
  commodity_symbol VARCHAR UNIQUE,
  commodity_name VARCHAR,
  currency VARCHAR
);

INSERT INTO demo_app.commodities(commodity_symbol, commodity_name, currency)
VALUES ('BZUSD','Brent Crude Oil','USD'), ('GCUSD', 'Gold Futures', 'USD');

CREATE TABLE IF NOT EXISTS demo_app.company_income(
  id SERIAL PRIMARY KEY,
  symbol VARCHAR,
  earning_calendar VARCHAR [],
  income_statement jsonb
);

CREATE TABLE IF NOT EXISTS demo_app.company_balance(
  id SERIAL PRIMARY KEY,
  symbol VARCHAR,
  earning_calendar VARCHAR [],
  balance_sheet jsonb
);

CREATE TABLE IF NOT EXISTS demo_app.company_cashflow(
  id SERIAL PRIMARY KEY,
  symbol VARCHAR,
  earning_calendar VARCHAR [],
  cashflow jsonb
);