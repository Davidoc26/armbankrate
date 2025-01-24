extern crate core;

mod ardshinbank;
mod conversebank;
mod error;
mod evocabank;
mod idbank;
mod inecobank;
pub mod sort;
mod unibank;

use std::collections::HashMap;
use std::fmt::Debug;

use crate::ardshinbank::Ardshinbank;
use crate::conversebank::Conversebank;
use crate::evocabank::Evocabank;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::Html;
use serde::Serialize;
use std::str::FromStr;

use crate::error::Error;
use crate::Error::BankParseFail;

use crate::idbank::Idbank;
use crate::inecobank::Inecobank;

use crate::unibank::Unibank;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::builder().user_agent("Some").build().unwrap());

pub async fn parse<T: ToString>(banks: &[T]) -> Result<Vec<Bank>, Error> {
    let mut banks: Vec<Bank> = banks
        .iter()
        .map(|bank| bank_from_str(bank.to_string()))
        .collect::<Result<Vec<Bank>, Error>>()?;

    parse_banks(&mut banks).await;

    Ok(banks)
}

pub async fn parse_all() -> Result<Vec<Bank>, Error> {
    let mut banks: Vec<Bank> = get_bank_vec();

    parse_banks(&mut banks).await;

    Ok(banks)
}

pub async fn parse_json<T: ToString>(banks: &[T]) -> Result<String, Error> {
    let mut banks: Vec<Bank> = banks
        .iter()
        .map(|bank| bank_from_str(bank.to_string()))
        .collect::<Result<Vec<Bank>, Error>>()?;

    parse_banks(&mut banks).await;

    json_from(&banks)
}

pub async fn parse_all_json() -> Result<String, Error> {
    let mut banks = get_bank_vec();

    parse_banks(&mut banks).await;

    json_from(&banks)
}

async fn parse_banks(banks: &mut Vec<Bank>) {
    let futures = FuturesUnordered::new();
    for bank in banks {
        futures.push(bank.parse());
    }
    futures.collect::<Vec<Result<_, _>>>().await;
}

fn json_from(banks: &Vec<Bank>) -> Result<String, Error> {
    let mut bank_map: HashMap<&String, &Bank> = HashMap::with_capacity(banks.len());

    for bank in banks {
        let bank_name = bank.get_name();
        bank_map.insert(bank_name, bank);
    }

    Ok(serde_json::to_string(&bank_map)?)
}

fn bank_from_str<T: ToString>(s: T) -> Result<Bank, Error> {
    let s = s.to_string().to_lowercase();

    match s.as_str() {
        "ardshinbank" => Ok(Ardshinbank::default().into()),
        "conversebank" => Ok(Conversebank::default().into()),
        "evocabank" => Ok(Evocabank::default().into()),
        "idbank" => Ok(Idbank::default().into()),
        "inecobank" => Ok(Inecobank::default().into()),
        "unibank" => Ok(Unibank::default().into()),
        _ => Err(Error::BankNotFound(s)),
    }
}

fn get_bank_vec() -> Vec<Bank> {
    vec![
        Unibank::default().into(),
        Conversebank::default().into(),
        Idbank::default().into(),
        Evocabank::default().into(),
        Inecobank::default().into(),
        Ardshinbank::default().into(),
    ]
}

#[async_trait]
#[enum_dispatch]
pub trait BankImpl: Send {
    async fn parse(&mut self) -> Result<(), Error> {
        let url = self.get_url();

        let response = CLIENT.get(url).send().await?.text().await?;
        let document = Html::parse_document(&response);

        self.parse_cash(&document)?;
        self.parse_no_cash(&document)?;

        Ok(())
    }

    fn parse_cash(&mut self, document: &Html) -> Result<(), Error>;
    fn parse_no_cash(&mut self, document: &Html) -> Result<(), Error>;

    fn cash_currencies(&self) -> &CurrencyBody;
    fn no_cash_currencies(&self) -> &CurrencyBody;

    fn get_name(&self) -> &String;
    fn get_url(&self) -> &String;
}

#[derive(Default, Debug, Serialize)]
pub struct Currency {
    name: CurrencyName,
    buy: Option<f64>,
    sell: Option<f64>,
}

#[derive(Default, Debug)]
struct BankBody {
    name: String,
    url: String,
}

impl Currency {
    pub fn new(name: CurrencyName, buy: Option<f64>, sell: Option<f64>) -> Self {
        Self { name, buy, sell }
    }

    pub fn buy(&self) -> &Option<f64> {
        &self.buy
    }

    pub fn sell(&self) -> &Option<f64> {
        &self.sell
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum CurrencyName {
    USD,
    GBP,
    EUR,
    RUB,
}

impl Default for CurrencyName {
    fn default() -> Self {
        Self::RUB
    }
}

impl FromStr for CurrencyName {
    type Err = error::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_uppercase();

        match value.as_str() {
            "USD" => Ok(CurrencyName::USD),
            "RUR" | "RUB" => Ok(CurrencyName::RUB),
            "GBP" => Ok(CurrencyName::GBP),
            "EUR" => Ok(CurrencyName::EUR),
            _ => Err(Error::CurrencyNotFound(value)),
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct CurrencyBody {
    usd: Currency,
    gbp: Currency,
    eur: Currency,
    rub: Currency,
}

impl CurrencyBody {
    pub fn get_usd_rate(&self) -> &Currency {
        &self.usd
    }

    pub fn get_gbp_rate(&self) -> &Currency {
        &self.gbp
    }

    pub fn get_eur_rate(&self) -> &Currency {
        &self.eur
    }

    pub fn get_rub_rate(&self) -> &Currency {
        &self.rub
    }

    pub fn fill_from_currency(&mut self, currency: Currency) {
        match currency.name {
            CurrencyName::USD => self.usd = currency,
            CurrencyName::GBP => self.gbp = currency,
            CurrencyName::EUR => self.eur = currency,
            CurrencyName::RUB => self.rub = currency,
        }
    }
}

#[enum_dispatch(BankImpl)]
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Bank {
    Ardshinbank,
    Inecobank,
    Evocabank,
    Idbank,
    Conversebank,
    Unibank,
}

#[derive(Debug, Clone)]
pub enum CurrencyType {
    Cash,
    Noncash,
}
