-- Add down migration script here
DROP SCHEMA demo_app CASCADE;
DROP SCHEMA weblog CASCADE;
DROP TABLE weblog.session_table;
DROP TABLE weblog.user;
DROP TABLE weblog.who_is_login;
DROP TABLE demo_app.stock_sector;
DROP TABLE demo_app.stock_profile;
DROP TABLE demo_app.company;
DROP TABLE demo_app.stock_news;
DROP TABLE demo_app.fmp_news;
DROP TABLE demo_app.stock_price;
DROP TABLE demo_app.indices;
DROP TABLE demo_app.commodities;
DROP TABLE demo_app.company_income;
DROP TABLE demo_app.company_balance;
DROP TABLE demo_app.company_cashflow;