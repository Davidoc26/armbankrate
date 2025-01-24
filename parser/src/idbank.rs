use async_trait::async_trait;

use crate::{
    BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error, CLIENT,
};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Idbank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    cashless_currencies: CurrencyBody,
    #[serde(skip_serializing)]
    main_selector: Selector,
    #[serde(skip_serializing)]
    currency_name_regex: Regex,
    #[serde(skip_serializing)]
    currency_value_regex: Regex,
}

impl Default for Idbank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Idbank".to_string(),
                url: "https://idbank.am/en/rates/".to_string(),
            },
            cash_currencies: Default::default(),
            cashless_currencies: Default::default(),
            main_selector: Selector::parse("#\\.default > div.m-exchange > div.m-exchange__table > div > .m-exchange__table-cell:nth-child(1)").unwrap(),
            currency_name_regex: Regex::new(r"\d \w{3}").unwrap(),
            currency_value_regex: Regex::new(r"\d+\.?\d+").unwrap(),
        }
    }
}

#[async_trait]
impl BankImpl for Idbank {
    async fn parse(&mut self) -> Result<(), Error> {
        let cash_response = CLIENT
            .post(self.get_url())
            .form(&[("RATE_TYPE", "CASH")])
            .send()
            .await?
            .text()
            .await?;

        let cashless_response = CLIENT
            .post(self.get_url())
            .form(&[("RATE_TYPE", "NO_CASH")])
            .send()
            .await?
            .text()
            .await?;

        self.parse_cash(&Html::parse_document(&cash_response))?;
        self.parse_no_cash(&Html::parse_document(&cashless_response))?;

        Ok(())
    }

    fn parse_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.main_selector).skip(1).take(5) {
            let currency_name = match self.currency_name_regex.find(&element.inner_html()) {
                Some(matched) => {
                    match CurrencyName::from_str(matched.as_str().trim_start_matches("1 ")) {
                        Ok(name) => name,
                        Err(_) => continue,
                    }
                }
                None => continue,
            };

            let currency_buy: f64 = {
                let inner_html =
                    ElementRef::wrap(element.next_siblings().nth(1).ok_or(BankParseFail)?)
                        .ok_or(BankParseFail)?
                        .inner_html();
                match self.currency_value_regex.find(&inner_html) {
                    Some(matched) => matched.as_str().parse::<f64>().unwrap_or_default(),
                    None => Default::default(),
                }
            };

            let currency_sell: f64 = {
                let inner_html =
                    ElementRef::wrap(element.next_siblings().nth(3).ok_or(BankParseFail)?)
                        .ok_or(BankParseFail)?
                        .inner_html();
                match self.currency_value_regex.find(&inner_html) {
                    Some(matched) => matched.as_str().parse::<f64>().unwrap_or_default(),
                    None => Default::default(),
                }
            };

            let currency = Currency::new(currency_name, Some(currency_buy), Some(currency_sell));
            self.cash_currencies.fill_from_currency(currency);
        }
        Ok(())
    }

    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.main_selector).skip(1) {
            let currency_name = match self.currency_name_regex.find(&element.inner_html()) {
                Some(matched) => {
                    match CurrencyName::from_str(matched.as_str().trim_start_matches("1 ")) {
                        Ok(name) => name,
                        Err(_) => continue,
                    }
                }
                None => continue,
            };

            let currency_buy: f64 = {
                let inner_html =
                    ElementRef::wrap(element.next_siblings().nth(1).ok_or(BankParseFail)?)
                        .ok_or(BankParseFail)?
                        .inner_html();
                match self.currency_value_regex.find(&inner_html) {
                    Some(matched) => matched.as_str().parse::<f64>().unwrap_or_default(),
                    None => Default::default(),
                }
            };

            let currency_sell: f64 = {
                let inner_html =
                    ElementRef::wrap(element.next_siblings().nth(3).ok_or(BankParseFail)?)
                        .ok_or(BankParseFail)?
                        .inner_html();
                match self.currency_value_regex.find(&inner_html) {
                    Some(matched) => matched.as_str().parse::<f64>().unwrap_or_default(),
                    None => Default::default(),
                }
            };

            let currency = Currency::new(currency_name, Some(currency_buy), Some(currency_sell));
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
