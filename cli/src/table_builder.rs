use crate::{Colorized, CurrencyName, CurrencyType};
use armbankrate_parser::{Bank, BankImpl, CurrencyBody};
use colored::Colorize;
use std::cell::RefCell;

use tabled::builder::Builder;
use tabled::object::Segment;
use tabled::{Alignment, Concat, Header, Modify, Style, Table};

pub(crate) struct TableBuilder {
    banks: RefCell<Vec<Bank>>,
    currency_type: CurrencyType,
    sort: Option<CurrencyName>,
    builder: RefCell<Builder>,
}

impl TableBuilder {
    pub(crate) fn new(
        banks: Vec<Bank>,
        currency_type: CurrencyType,
        sort: Option<CurrencyName>,
    ) -> Self {
        Self {
            banks: RefCell::new(banks),
            currency_type,
            sort,
            builder: RefCell::new(Builder::default()),
        }
    }

    pub(crate) fn build(&self) -> String {
        match self.currency_type {
            CurrencyType::All => self
                .build_cash()
                .with(Style::ascii())
                .with(Concat::vertical(self.build_no_cash()))
                .to_string(),
            CurrencyType::Cash => self.build_cash().to_string(),
            CurrencyType::Noncash => self.build_no_cash().to_string(),
        }
    }

    fn build_table(&self, currency_header: CurrencyHeader) -> Table {
        // Reset builder when building
        let builder = self.builder.take();

        let header = match currency_header {
            CurrencyHeader::Cash => "CASH".bold().bright_green().to_string(),
            CurrencyHeader::Noncash => "NON-CASH".bold().bright_green().to_string(),
        };

        builder
            .build()
            .with(Header(header))
            .with(
                Modify::new(Segment::all())
                    .with(Alignment::center())
                    .with(Alignment::center()),
            )
            .with(Style::extended())
    }

    fn build_cash(&self) -> Table {
        self.prepare_columns();

        self.sort_banks(armbankrate_parser::CurrencyType::Cash);

        for bank in self.banks.borrow().iter() {
            let currencies: &CurrencyBody = bank.cash_currencies();
            self.add_column(bank, currencies);
        }

        self.build_table(CurrencyHeader::Cash)
    }

    fn build_no_cash(&self) -> Table {
        self.prepare_columns();

        self.sort_banks(armbankrate_parser::CurrencyType::Noncash);

        for bank in self.banks.borrow().iter() {
            let currencies: &CurrencyBody = bank.no_cash_currencies();
            self.add_column(bank, currencies);
        }

        self.build_table(CurrencyHeader::Noncash)
    }

    fn add_column(&self, bank: &Bank, currencies: &CurrencyBody) {
        self.builder.borrow_mut().add_record([
            &bank.get_name().bright_yellow().bold().to_string(),
            &currencies.get_usd_rate().colorized(),
            &currencies.get_eur_rate().colorized(),
            &currencies.get_rub_rate().colorized(),
            &currencies.get_gbp_rate().colorized(),
        ]);
    }

    fn prepare_columns(&self) {
        self.builder.borrow_mut().set_columns([
            "Bank".colorized(),
            "USD".colorized(),
            "EUR".colorized(),
            "RUB".colorized(),
            "GBP".colorized(),
        ]);
    }

    fn sort_banks(&self, currency_type: armbankrate_parser::CurrencyType) {
        if let Some(sort) = &self.sort {
            let mut banks = self.banks.borrow_mut();
            armbankrate_parser::sort::sort_banks(&mut banks, &sort.to_sort_data(currency_type))
        }
    }
}

enum CurrencyHeader {
    Cash,
    Noncash,
}
