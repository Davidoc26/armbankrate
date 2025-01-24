mod table_builder;

#[macro_use]
extern crate enum_display_derive;

use crate::table_builder::TableBuilder;
use anyhow::{Context, Result};
use armbankrate_parser::sort::OrderType;
use armbankrate_parser::Currency;
use clap::{ArgEnum, Parser, Subcommand};
use colored::Colorize;
use std::fmt::Display;

static ERR_MSG: &str = "Something went wrong while receiving bank rates";

#[derive(ArgEnum, Display, Debug, Clone, PartialEq)]
enum Banks {
    All,
    Ardshinbank,
    Inecobank,
    Evocabank,
    Idbank,
    Conversebank,
    Unibank,
}

#[derive(ArgEnum, Display, Debug, Clone)]
enum CurrencyType {
    All,
    Cash,
    Noncash,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Displays the rates of all banks or selected ones (use parse -h to see the options)
    Parse {
        #[clap(value_enum, default_value_t = CurrencyType::All)]
        currency_type: CurrencyType,
        #[clap(value_parser)]
        banks: Vec<Banks>,
        #[clap(value_enum, long, short)]
        sort: Option<CurrencyName>,
    },
    /// Parses banks and outputs as JSON (banks can be selected)
    Json {
        #[clap(value_parser)]
        banks: Vec<Banks>,
    },
}

#[derive(Parser, Debug)]
#[clap(name = "armbankrate")]
#[clap(author = "David Eritsyan <dav.eritsyan@gmail.com>")]
#[clap(version = "0.1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Output program execution time
    #[clap(short, long, action)]
    time: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let start = std::time::Instant::now();

    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            banks,
            currency_type,
            sort,
        } => handle_parse(banks, currency_type, sort).await?,
        Commands::Json { banks } => handle_json(banks).await?,
    };

    if cli.time {
        println!("Time elapsed: {:?}", start.elapsed());
    }
    std::process::exit(0);
}

pub trait Colorized {
    fn colorized(&self) -> String;
}

impl Colorized for &str {
    fn colorized(&self) -> String {
        self.bright_cyan().bold().to_string()
    }
}

impl Colorized for Currency {
    fn colorized(&self) -> String {
        format!(
            "{} / {}",
            self.buy().unwrap_or_default().to_string().bright_green(),
            self.sell().unwrap_or_default().to_string().bright_red()
        )
    }
}

async fn handle_parse(
    banks: Vec<Banks>,
    currency_type: CurrencyType,
    sort_by: Option<CurrencyName>,
) -> Result<()> {
    match banks.is_empty() || banks.contains(&Banks::All) {
        true => {
            let banks = armbankrate_parser::parse_all()
                .await
                .with_context(|| ERR_MSG)?;

            let table = TableBuilder::new(banks, currency_type, sort_by).build();
            println!("{}", table);
        }
        false => {
            let banks = armbankrate_parser::parse(&banks)
                .await
                .with_context(|| ERR_MSG)?;
            let table = TableBuilder::new(banks, currency_type, sort_by).build();
            println!("{}", table);
        }
    }

    Ok(())
}

async fn handle_json(banks: Vec<Banks>) -> Result<()> {
    match banks.is_empty() {
        true => {
            let banks_json = armbankrate_parser::parse_all_json()
                .await
                .with_context(|| ERR_MSG)?;
            println!("{banks_json}");
        }
        false => {
            let banks_json = armbankrate_parser::parse_json(&banks)
                .await
                .with_context(|| ERR_MSG)?;
            println!("{banks_json}");
        }
    }

    Ok(())
}

#[derive(ArgEnum, Display, Debug, Clone)]
enum CurrencyName {
    UsdBuy,
    UsdSell,
    GbpBuy,
    GbpSell,
    EurBuy,
    EurSell,
    RubBuy,
    RubSell,
}

impl CurrencyName {
    fn to_sort_data(
        &self,
        currency_type: armbankrate_parser::CurrencyType,
    ) -> armbankrate_parser::sort::SortData {
        match self {
            CurrencyName::UsdBuy => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::USD,
                OrderType::Buy,
            ),
            CurrencyName::UsdSell => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::USD,
                OrderType::Sell,
            ),
            CurrencyName::GbpBuy => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::GBP,
                OrderType::Buy,
            ),
            CurrencyName::GbpSell => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::GBP,
                OrderType::Sell,
            ),
            CurrencyName::EurBuy => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::EUR,
                OrderType::Buy,
            ),
            CurrencyName::EurSell => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::EUR,
                OrderType::Sell,
            ),
            CurrencyName::RubBuy => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::RUB,
                OrderType::Buy,
            ),
            CurrencyName::RubSell => armbankrate_parser::sort::SortData::new(
                currency_type,
                armbankrate_parser::CurrencyName::RUB,
                OrderType::Sell,
            ),
        }
    }
}
