use crate::{Bank, BankImpl, Currency, CurrencyBody, CurrencyName, CurrencyType};
use std::cmp::Ordering;

pub fn sort_banks(banks: &mut [Bank], sort_data: &SortData) {
    banks.sort_by(|a, b| {
        let (a_currencies, b_currencies): (&CurrencyBody, &CurrencyBody) =
            currencies_by_type(a, b, &sort_data.currency_type);

        match sort_data.currency_name {
            CurrencyName::USD => compare(
                a_currencies.get_usd_rate(),
                b_currencies.get_usd_rate(),
                &sort_data.order_type,
            ),
            CurrencyName::GBP => compare(
                a_currencies.get_gbp_rate(),
                b_currencies.get_gbp_rate(),
                &sort_data.order_type,
            ),
            CurrencyName::EUR => compare(
                a_currencies.get_eur_rate(),
                b_currencies.get_eur_rate(),
                &sort_data.order_type,
            ),
            CurrencyName::RUB => compare(
                a_currencies.get_rub_rate(),
                b_currencies.get_rub_rate(),
                &sort_data.order_type,
            ),
        }
        .reverse()
    });
}

fn currencies_by_type<'a>(
    a: &'a Bank,
    b: &'a Bank,
    currency_type: &CurrencyType,
) -> (&'a CurrencyBody, &'a CurrencyBody) {
    match currency_type {
        CurrencyType::Cash => (a.cash_currencies(), b.cash_currencies()),
        CurrencyType::Noncash => (a.no_cash_currencies(), b.no_cash_currencies()),
    }
}

fn compare(a: &Currency, b: &Currency, order_type: &OrderType) -> Ordering {
    let (a, b) = match order_type {
        OrderType::Buy => (a.buy(), b.buy()),
        OrderType::Sell => (a.sell(), b.sell()),
    };

    a.unwrap_or_default().total_cmp(&b.unwrap_or_default())
}

pub struct SortData {
    currency_type: CurrencyType,
    currency_name: CurrencyName,
    order_type: OrderType,
}

impl SortData {
    pub fn new(
        currency_type: CurrencyType,
        currency_name: CurrencyName,
        order_type: OrderType,
    ) -> Self {
        Self {
            currency_type,
            currency_name,
            order_type,
        }
    }
}

pub enum OrderType {
    Buy,
    Sell,
}
