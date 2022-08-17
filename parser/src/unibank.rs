use crate::{BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Unibank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    cashless_currencies: CurrencyBody,
    #[serde(skip_serializing)]
    cash_selector: Selector,
    #[serde(skip_serializing)]
    no_cash_selector: Selector,
}

impl Default for Unibank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Unibank".to_string(),
                url: "https://www.unibank.am/".to_string(),
            },
            cash_currencies: Default::default(),
            cashless_currencies: Default::default(),
            cash_selector: Selector::parse("#Cash > div.pane__body > ul:nth-child(2) > li:nth-child(3n+1)").unwrap(),
            no_cash_selector: Selector::parse("#Noncash > div.pane__body > ul > li:nth-child(3n+1)").unwrap(),
        }
    }
}

impl BankImpl for Unibank {
    fn parse_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.cash_selector).take(4) {
            let currency_name = {
                let value = ElementRef::wrap(element.children().next().ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .inner_html();
                match CurrencyName::from_str(&value) {
                    Ok(name) => name,
                    Err(_) => continue,
                }
            };

            let currency_buy: f64 = {
                ElementRef::wrap(
                    element
                        .next_siblings()
                        .nth(1)
                        .ok_or(BankParseFail)?
                        .first_child()
                        .ok_or(BankParseFail)?
                )
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency_sell: f64 = {
                ElementRef::wrap(
                    element
                        .next_siblings()
                        .nth(3)
                        .ok_or(BankParseFail)?
                        .first_child()
                        .ok_or(BankParseFail)?
                )
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency = Currency::new(currency_name, Some(currency_buy), Some(currency_sell));
            self.cash_currencies.fill_from_currency(currency);
        }

        Ok(())
    }

    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error> {
        for element in document.select(&self.no_cash_selector) {
            let currency_name = {
                let value = ElementRef::wrap(element.children().next().ok_or(BankParseFail)?)
                    .ok_or(BankParseFail)?
                    .inner_html();
                match CurrencyName::from_str(&value) {
                    Ok(name) => name,
                    Err(_) => continue,
                }
            };

            let currency_buy: f64 = {
                ElementRef::wrap(
                    element
                        .next_siblings()
                        .nth(1)
                        .ok_or(BankParseFail)?
                        .first_child()
                        .ok_or(BankParseFail)?
                )
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
            };

            let currency_sell: f64 = {
                ElementRef::wrap(
                    element
                        .next_siblings()
                        .nth(3)
                        .ok_or(BankParseFail)?
                        .first_child()
                        .ok_or(BankParseFail)?
                )
                    .ok_or(BankParseFail)?
                    .inner_html()
                    .parse::<f64>()?
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
