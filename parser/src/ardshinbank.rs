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
                url: "https://website-api.ardshinbank.am/currency".to_string(),
            },
            cash_currencies: Default::default(),
            no_cash_currencies: Default::default(),
        }
    }
}

#[async_trait]
impl BankImpl for Ardshinbank {
    async fn parse(&mut self) -> Result<(), Error> {
        let response = CLIENT
            .get(self.get_url())
            .send()
            .await?
            .json::<HashMap<String, Value>>()
            .await?;
        let cash_currencies = response["data"]["currencies"]["cash"]
            .as_array()
            .ok_or(BankParseFail)?;
        let no_cash_currencies = response["data"]["currencies"]["no_cash"]
            .as_array()
            .ok_or(BankParseFail)?;

        for currency in cash_currencies {
            let currency_name =
                CurrencyName::from_str(currency["type"].as_str().ok_or(BankParseFail)?);
            let currency_name = match currency_name {
                Ok(name) => name,
                Err(_) => continue,
            };

            let buy = serde_json::from_str::<f64>(currency["buy"].as_str().ok_or(BankParseFail)?)?;
            let sell =
                serde_json::from_str::<f64>(currency["sell"].as_str().ok_or(BankParseFail)?)?;
            let currency = Currency::new(currency_name, buy.into(), sell.into());
            self.cash_currencies.fill_from_currency(currency);
        }

        for currency in no_cash_currencies {
            let currency_name =
                CurrencyName::from_str(currency["type"].as_str().ok_or(BankParseFail)?);
            let currency_name = match currency_name {
                Ok(name) => name,
                Err(_) => continue,
            };

            let buy = serde_json::from_str::<f64>(currency["buy"].as_str().ok_or(BankParseFail)?)?;
            let sell =
                serde_json::from_str::<f64>(currency["sell"].as_str().ok_or(BankParseFail)?)?;
            let currency = Currency::new(currency_name, buy.into(), sell.into());
            self.no_cash_currencies.fill_from_currency(currency);
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
        &self.no_cash_currencies
    }

    fn get_name(&self) -> &String {
        &self.body.name
    }

    fn get_url(&self) -> &String {
        &self.body.url
    }
}
