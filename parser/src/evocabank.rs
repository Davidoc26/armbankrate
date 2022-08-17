use crate::{BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Evocabank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    cashless_currencies: CurrencyBody,
    #[serde(skip_serializing)]
    cash_selector: Selector,
    #[serde(skip_serializing)]
    no_cash_selector: Selector,
    #[serde(skip_serializing)]
    span_selector: Selector,
}

impl Default for Evocabank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Evocabank".to_string(),
                url: "https://www.evoca.am/".to_string(),
            },
            cash_currencies: Default::default(),
            cashless_currencies: Default::default(),
            cash_selector: Selector::parse("#tab-1 > div > div.exchange > div > div.exchange__box > div > div > table > tbody > tr").unwrap(),
            no_cash_selector: Selector::parse("#tab-2 > div > div.exchange > div > div.exchange__box > div > div > table > tbody > tr").unwrap(),
            span_selector: Selector::parse("span").unwrap(),
        }
    }
}

impl BankImpl for Evocabank {
    fn parse_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.cash_selector).take(4) {
            let currency_name = element.select(&self.span_selector).next().ok_or(BankParseFail)?.inner_html();

            let currency_name = match CurrencyName::from_str(&currency_name) {
                Ok(currency_name) => currency_name,
                Err(_) => continue,
            };

            let currency_buy = ElementRef::wrap(element.children().nth(3).ok_or(BankParseFail)?)
                .ok_or(BankParseFail)?
                .inner_html()
                .trim_start()
                .trim_end()
                .parse::<f64>()?;

            let currency_sell = ElementRef::wrap(element.children().nth(5).ok_or(BankParseFail)?)
                .ok_or(BankParseFail)?
                .inner_html()
                .trim_start()
                .trim_end()
                .parse::<f64>()?;

            let currency = Currency::new(
                currency_name,
                Some(currency_buy),
                Some(currency_sell),
            );

            self.cash_currencies.fill_from_currency(currency);
        }
        Ok(())
    }

    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.no_cash_selector).take(4) {
            let currency_name = element.select(&self.span_selector).next().ok_or(BankParseFail)?.inner_html();

            let currency_name = match CurrencyName::from_str(&currency_name) {
                Ok(currency_name) => currency_name,
                Err(_) => continue,
            };

            let currency_buy = ElementRef::wrap(element.children().nth(3).ok_or(BankParseFail)?)
                .ok_or(BankParseFail)?
                .inner_html()
                .trim_start()
                .trim_end()
                .parse::<f64>()?;

            let currency_sell = ElementRef::wrap(element.children().nth(5).ok_or(BankParseFail)?)
                .ok_or(BankParseFail)?
                .inner_html()
                .trim_start()
                .trim_end()
                .parse::<f64>()?;

            let currency = Currency::new(
                currency_name,
                Some(currency_buy),
                Some(currency_sell),
            );
            self.cashless_currencies.fill_from_currency(currency);
        }
        Ok(())
    }

    fn cash_currencies(&self) -> &CurrencyBody {
        &self.cash_currencies
    }

    fn no_cash_currencies(&self) -> &CurrencyBody {
        &self.cashless_currencies
    }

    fn get_name(&self) -> &String {
        &self.body.name
    }

    fn get_url(&self) -> &String {
        &self.body.url
    }
}
