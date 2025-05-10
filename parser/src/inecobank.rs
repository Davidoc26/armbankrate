use crate::{
    BankBody, BankImpl, BankParseFail, Currency, CurrencyBody, CurrencyName, Error, CLIENT,
};
use async_trait::async_trait;
use scraper::Html;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Inecobank {
    #[serde(skip_serializing)]
    body: BankBody,
    cash_currencies: CurrencyBody,
    cashless_currencies: CurrencyBody,
}

impl Default for Inecobank {
    fn default() -> Self {
        Self {
            body: BankBody {
                name: "Inecobank",
                url: "https://www.inecobank.am/api/rates/",
            },
            cash_currencies: Default::default(),
            cashless_currencies: Default::default(),
        }
    }
}

#[async_trait]
impl BankImpl for Inecobank {
    async fn parse(&mut self) -> Result<(), Error> {
        let response = CLIENT
            .get(self.get_url())
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;

        for item in response["items"].as_array().ok_or(BankParseFail)? {
            let code = item["code"].as_str().ok_or(BankParseFail)?;
            let cash = item["cash"].as_object().ok_or(BankParseFail)?;
            let cashless = item["cashless"].as_object().ok_or(BankParseFail)?;
            let currency_name = CurrencyName::from_str(code);
            let currency_name = match currency_name {
                Ok(name) => name,
                Err(_) => continue,
            };

            // Fill cash
            let currency = Currency::new(
                currency_name.clone(),
                cash["buy"].as_f64(),
                cash["sell"].as_f64(),
            );
            self.cash_currencies.fill_from_currency(currency);

            // Fill cashless
            let currency = Currency::new(
                currency_name,
                cashless["buy"].as_f64(),
                cashless["sell"].as_f64(),
            );
            self.cashless_currencies.fill_from_currency(currency);
        }

        Ok(())
    }

    fn parse_cash(&mut self, _document: &Html) -> Result<(), Error> {
        unreachable!()
    }

    fn parse_no_cash(&mut self, _document: &Html) -> Result<(), Error> {
        unreachable!()
    }

    fn cash_currencies(&self) -> &CurrencyBody {
        &self.cash_currencies
    }

    fn no_cash_currencies(&self) -> &CurrencyBody {
        &self.cashless_currencies
    }

    fn get_name(&self) -> &str {
        self.body.name
    }

    fn get_url(&self) -> &str {
        self.body.url
    }
}
