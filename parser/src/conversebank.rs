use crate::{BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error};
use async_trait::async_trait;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Conversebank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    cashless_currencies: CurrencyBody,
    #[serde(skip_serializing)]
    main_selector: Selector,
}

impl Conversebank {
    fn parse_currency_name_from_element(
        &self,
        element: &ElementRef,
    ) -> Result<CurrencyName, Error> {
        let name = ElementRef::wrap(element.children().nth(1).ok_or(BankParseFail)?)
            .ok_or(BankParseFail)?
            .inner_html();

        CurrencyName::from_str(&name)
    }

    fn parse_currency_from_element(&self, element: &ElementRef, nth: usize) -> Result<f64, Error> {
        Ok(
            ElementRef::wrap(element.children().nth(nth).ok_or(BankParseFail)?)
                .ok_or(BankParseFail)?
                .inner_html()
                .parse::<f64>()?,
        )
    }
}

impl Default for Conversebank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Conversebank".to_string(),
                url: "https://www.conversebank.am/ru/exchange-rate/".to_string(),
            },
            cash_currencies: Default::default(),
            cashless_currencies: Default::default(),
            main_selector: Selector::parse(
                "#main_static_content > table:nth-child(5) > tbody > tr",
            )
            .unwrap(),
        }
    }
}

#[async_trait]
impl BankImpl for Conversebank {
    fn parse_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.main_selector).skip(2).take(4) {
            let currency_name = match self.parse_currency_name_from_element(&element) {
                Ok(currency_name) => currency_name,
                Err(_) => continue,
            };

            let currency_buy = self.parse_currency_from_element(&element, 7)?;
            let currency_sell = self.parse_currency_from_element(&element, 9)?;

            let currency = Currency::new(currency_name, Some(currency_buy), Some(currency_sell));
            self.cash_currencies.fill_from_currency(currency);
        }

        Ok(())
    }

    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.main_selector).skip(2).take(4) {
            let currency_name = match self.parse_currency_name_from_element(&element) {
                Ok(currency_name) => currency_name,
                Err(_) => continue,
            };

            let currency_buy = self.parse_currency_from_element(&element, 11)?;
            let currency_sell = self.parse_currency_from_element(&element, 13)?;

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
