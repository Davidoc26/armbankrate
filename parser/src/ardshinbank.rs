use crate::{BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error};
use async_trait::async_trait;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Ardshinbank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    no_cash_currencies: CurrencyBody,
}

impl Default for Ardshinbank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Ardshinbank".to_string(),
                url: "https://www.ardshinbank.am".to_string(),
            },
            cash_currencies: Default::default(),
            no_cash_currencies: Default::default(),
        }
    }
}

#[async_trait]
impl BankImpl for Ardshinbank {
    fn parse_cash(&mut self, document: &Html) -> Result<(), Error> {
        let selector = Selector::parse(r#"#cash > table > tbody > tr > td.tg-cod"#).unwrap();
        let span_selector = Selector::parse("span").unwrap();

        for element in document.select(&selector).take(4) {
            let currency_name = {
                element
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .trim_start_matches('\n')
                    .to_string()
            };

            let currency_buy = {
                ElementRef::wrap(element.next_siblings().nth(1).ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency_sell = {
                ElementRef::wrap(element.next_siblings().nth(3).ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency = Currency::new(
                CurrencyName::from_str(&currency_name).unwrap_or_default(),
                currency_buy.into(),
                currency_sell.into(),
            );

            self.cash_currencies.fill_from_currency(currency);
        }
        Ok(())
    }

    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error> {
        let selector = Selector::parse(r#"#no-cash > table > tbody > tr > td.tg-cod"#).unwrap();
        let span_selector = Selector::parse("span").unwrap();

        for element in document.select(&selector).take(4) {
            let currency_name = {
                element
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .trim_start_matches('\n')
                    .to_string()
            };

            let currency_name = match CurrencyName::from_str(&currency_name) {
                Ok(name) => name,
                Err(_) => continue,
            };

            let currency_buy = {
                ElementRef::wrap(element.next_siblings().nth(1).ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency_sell = {
                ElementRef::wrap(element.next_siblings().nth(3).ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .select(&span_selector)
                    .next()
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency = Currency::new(
                currency_name,
                Some(currency_buy),
                Some(currency_sell),
            );

            self.no_cash_currencies.fill_from_currency(currency);
        }
        Ok(())
    }

    fn cash_currencies(&self) -> &CurrencyBody {
        &self.cash_currencies
    }

    fn no_cash_currencies(&self) -> &CurrencyBody {
        &self.no_cash_currencies
    }

    fn get_name(&self) -> &String {
        &self.body.name
    }

    fn get_url(&self) -> &String {
        &self.body.url
    }
}
