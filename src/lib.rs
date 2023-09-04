mod abi;
mod pb;
mod utils;

use std::str::FromStr;

use crate::utils::{
    helper::{append_0x, get_sorted_token},
    pricing::get_price_y,
    rpc::get_token_data,
};
use abi::abi::{
    dexcandlesv2_factory as dexcandlesv2_factory_events,
    dexcandlesv2_pair as dexcandlesv2_pair_events,
};
use bigdecimal::ToPrimitive;
use substreams_entity_change::{pb::entity::EntityChanges, tables::Tables};

use pb::traderjoe::dexcandlesv2 as dexcandlesV2;
use substreams::{
    scalar::{BigDecimal, BigInt},
    store::{DeltaProto, Deltas, StoreNew, StoreSet},
    Hex,
};
use substreams_ethereum::{pb::eth, Event};

use substreams::{
    errors::Error,
    log,
    store::{StoreGet, StoreGetProto, StoreSetProto},
};
use utils::{
    constants::{CANDLESTICK_PERIODS, DEXCANDLES_FACTORY},
    helper::get_sorted_price,
};

#[substreams::handlers::map]
pub fn map_pairs_created(block: eth::v2::Block) -> Result<dexcandlesV2::Pairs, Error> {
    Ok(dexcandlesV2::Pairs {
        pairs: block
            .events::<dexcandlesv2_factory_events::events::LbPairCreated>(&[&DEXCANDLES_FACTORY])
            .map(|(pair, log)| {
                log::info!("New Pair Created 🚀🚀 ");

                let token_x_data = get_token_data(&pair.token_x);
                let token_y_data = get_token_data(&pair.token_y);

                dexcandlesV2::Pair {
                    address: append_0x(&Hex(pair.lb_pair).to_string()),
                    token_x: Some(dexcandlesV2::Token {
                        address: append_0x(&Hex(pair.token_x).to_string()),
                        decimal: token_x_data.2,
                        symbol: token_x_data.1,
                        name: token_x_data.0,
                    }),
                    token_y: Some(dexcandlesV2::Token {
                        address: append_0x(&Hex(pair.token_y).to_string()),
                        decimal: token_y_data.2,
                        symbol: token_y_data.1,
                        name: token_y_data.0,
                    }),
                    bin_step: pair.bin_step.to_string(),
                    block_number: block.number,
                    timestamp: block.timestamp_seconds(),
                    tx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                    log_index: log.index(),
                }
            })
            .collect(),
    })
}

#[substreams::handlers::map]
pub fn map_swaps(block: eth::v2::Block) -> Result<dexcandlesV2::Swaps, Error> {
    let mut swaps: Vec<dexcandlesV2::Swap> = Vec::new();
    for log in block.logs() {
        if let Some(swap_event) = dexcandlesv2_pair_events::events::Swap::match_and_decode(log) {
            log::info!("Swap Event Found");

            swaps.push(dexcandlesV2::Swap {
                pair_address: append_0x(&Hex(&log.address()).to_string()),
                amounts_in: swap_event.amounts_in.to_vec(),
                amounts_out: swap_event.amounts_out.to_vec(),
                id: swap_event.id.to_string(),
                block_number: block.number,
                timestamp: block.timestamp_seconds(),
                tx_hash: append_0x(&Hex(&log.receipt.transaction.hash).to_string()),
                log_index: log.index(),
            })
        }
    }
    Ok(dexcandlesV2::Swaps { swaps })
}

#[substreams::handlers::store]
pub fn store_pairs(i: dexcandlesV2::Pairs, o: StoreSetProto<dexcandlesV2::Pair>) {
    for pair in i.pairs {
        o.set(0, format!("Pair: {}", pair.address), &pair);
    }
}

// #[substreams::handlers::store]
// pub fn store_totals(i: dexcandlesV2::Swaps, o: StoreAddBigInt) {
//     for swap in i.swaps {
//         o.set(0, format!("TokenX: {}", pair.address), &pair);
//         o.set(0, format!("TokenY: {}", pair.address), &pair);
//     }
// }

#[substreams::handlers::store]
pub fn store_candles(
    pair: StoreGetProto<dexcandlesV2::Pair>,
    swap: dexcandlesV2::Swaps,
    o: StoreSetProto<dexcandlesV2::Candle>,
) {
    for s in swap.swaps {
        log::info!("Candle Found - 1");

        let pair_address = s.pair_address.to_lowercase();

        log::info!(&pair_address);

        let pairs = pair.get_last(format!("Pair: {}", pair_address));

        log::info!("Candle Found - 2");

        match pairs {
            Some(p) => {
                log::info!("Candle Found - 3");

                let token_x = p.token_x.unwrap();
                let token_y = p.token_y.unwrap();

                let token_x_address = token_x.address;
                let token_y_address = token_y.address;

                let token_x_decimals = token_x.decimal;
                let token_y_decimals = token_y.decimal;

                let token0 = get_sorted_token(&token_x_address, &token_y_address);
                let token1 = get_sorted_token(&token_y_address, &token_x_address);

                let price_y = get_price_y(
                    BigInt::from_str(p.bin_step.as_str()).unwrap(),
                    BigInt::from_str(s.id.as_str()).unwrap().to_i32(),
                    BigInt::from_str(token_x_decimals.as_str())
                        .unwrap()
                        .to_i32(),
                    BigInt::from_str(token_y_decimals.as_str())
                        .unwrap()
                        .to_i32(),
                );
                let price_x = BigDecimal::from_str("1").unwrap() / &price_y;

                log::info!(&price_x.to_string());

                let price =
                    get_sorted_price(&token0, &token1, &price_x.to_string(), &price_y.to_string());

                log::info!(&price.to_string());

                let mut tokens = Vec::from(s.amounts_in);
                tokens.extend_from_slice(&s.amounts_out);

                for candle_period in CANDLESTICK_PERIODS {
                    let timestamp = s.timestamp;
                    let time_id = timestamp / candle_period.to_u64().unwrap();
                    let time_params = &timestamp - (timestamp % candle_period.to_u64().unwrap());

                    let time_bytes = time_id.to_be_bytes();
                    let period_bytes = candle_period.to_be_bytes();

                    let mut candle_id = Vec::from(time_bytes);
                    candle_id.extend_from_slice(&period_bytes);
                    candle_id.extend_from_slice(&tokens);

                    o.set(
                        0,
                        format!("Candle: {}", Hex(&candle_id).to_string()),
                        &dexcandlesV2::Candle {
                            time: time_params.to_string(),
                            period: candle_period.to_string(),
                            last_block: s.timestamp.to_string(),
                            token0: token0.to_string(),
                            token1: token1.to_string(),
                            high: price.clone(),
                            open: price.clone(),
                            close: price.clone(),
                            low: price.clone(),
                        },
                    );
                }
            }
            None => log::info!("None Variant"),
        }
    }
}

#[substreams::handlers::store]
pub fn store_tokens(i: dexcandlesV2::Pairs, o: StoreSetProto<dexcandlesV2::Token>) {
    for pair in i.pairs {
        let token_x = &pair.token_x.unwrap();
        let token_y = &pair.token_y.unwrap();
        o.set(0, format!("Token: {}", &token_x.address), &token_x);
        o.set(0, format!("Token: {}", &token_y.address), &token_y);
    }
}

#[substreams::handlers::map]
pub fn graph_out(
    pairs: Deltas<DeltaProto<dexcandlesV2::Pair>>,
    tokens: Deltas<DeltaProto<dexcandlesV2::Token>>,
    candles: Deltas<DeltaProto<dexcandlesV2::Candle>>,
) -> Result<EntityChanges, Error> {
    let mut tables = Tables::new();
    //  token0TotalAmount :
    //  token1TotalAmount :
    //Set these fields from store_totals in Candles
    let entity_changes = tables.to_entity_changes();
    Ok(entity_changes)
}
