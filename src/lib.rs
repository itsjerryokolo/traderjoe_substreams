mod abi;
mod pb;
mod utils;

use std::str::FromStr;

use crate::utils::{
    helper::{
        append_0x, decode_x, decode_y, generate_key, get_sorted_amount0, get_sorted_amount1,
        get_sorted_price, get_sorted_token0, get_sorted_token1,
    },
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
    helper::{bigint_to_i32, get_amount_traded, reverse_bytes},
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
                    address: append_0x(&Hex(pair.lb_pair).to_string()).to_lowercase(),
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
                pair_address: append_0x(&Hex(&log.address()).to_string()).to_lowercase(),
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
        o.set(0, generate_key("Pair", &pair.address), &pair);
    }
}

// #[substreams::handlers::store]
// pub fn store_totals(i: StoreGetProto<dexcandlesV2::Candle>, o: StoreAddBigIntpop) {
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

        let pair_address = s.pair_address;

        log::info!(&pair_address);

        let pairs = pair.get_last(generate_key("Pair", &pair_address));

        log::info!("Candle Found - 2");

        match pairs {
            Some(p) => {
                log::info!("Candle Found - 3");

                let token_x = p.token_x.unwrap();
                let token_y = p.token_y.unwrap();

                let token_x_address = token_x.address;
                let token_y_address = token_y.address;

                log::info!("TokenX Address : {}", &token_x_address.to_string());
                log::info!("TokenY Address: {}", &token_y_address.to_string());

                let token_x_decimals = token_x.decimal;
                let token_y_decimals = token_y.decimal;

                let token0 = get_sorted_token0(&token_x_address, &token_y_address);
                let token1 = get_sorted_token1(&token_x_address, &token_y_address);

                let price_y = get_price_y(
                    bigint_to_i32(&p.bin_step),
                    bigint_to_i32(&s.id),
                    bigint_to_i32(&token_x_decimals),
                    bigint_to_i32(&token_y_decimals),
                );
                let price_x = BigDecimal::one() / &price_y;

                log::info!("PriceX : {}", &price_x);
                log::info!("PriceY : {}", &price_y);
                log::info!("isSortedToken0 : {}", &token0);
                log::info!("isSortedToken1 : {}", &token1);

                let price =
                    get_sorted_price(&token0, &token1, &price_x.to_string(), &price_y.to_string());

                log::info!(&price.to_string());

                for candle_period in CANDLESTICK_PERIODS {
                    let timestamp = s.timestamp;
                    let time_id = timestamp / candle_period.to_u64().unwrap();
                    let time_params = &timestamp - (timestamp % candle_period.to_u64().unwrap());

                    let time_bytes = time_id.to_ne_bytes();
                    let period_bytes = candle_period.to_ne_bytes();

                    let candle_id = format!(
                        "{}{}{}{}",
                        Hex(&time_bytes[0..4]).to_string(),
                        Hex(&period_bytes[0..4]).to_string(),
                        &token0.split("0x").last().unwrap(),
                        &token1.split("0x").last().unwrap(),
                    );

                    let amount_x = s.amounts_in.clone();
                    let amount_y = s.amounts_out.clone();

                    log::info!("AmountX : {:?}", &amount_x);
                    log::info!("AmountY : {:?}", &amount_y);

                    let reversed_amount_in_x_bytes = reverse_bytes(&amount_x);
                    let reversed_amount_in_y_bytes = reverse_bytes(&amount_x);
                    let reversed_amount_out_x_bytes = reverse_bytes(&amount_y);
                    let reversed_amount_out_y_bytes = reverse_bytes(&amount_y);

                    log::info!(
                        "reversed_amount_in_x_bytes : {:?}",
                        &reversed_amount_in_x_bytes
                    );
                    log::info!(
                        "reversed_amount_in_y_bytes : {:?}",
                        &reversed_amount_in_y_bytes
                    );
                    log::info!(
                        "reversed_amount_out_x_bytes : {:?}",
                        &reversed_amount_out_x_bytes
                    );
                    log::info!(
                        "reversed_amount_out_y_bytes : {:?}",
                        &reversed_amount_out_y_bytes
                    );

                    let amount_in_x = decode_x(reversed_amount_in_x_bytes);
                    let amount_in_y = decode_y(reversed_amount_in_y_bytes);
                    let amount_out_x = decode_x(reversed_amount_out_x_bytes);
                    let amount_out_y = decode_y(reversed_amount_out_y_bytes);

                    log::info!("amount_in_x : {}", &amount_in_x.to_string());
                    log::info!("amount_in_y : {}", &amount_in_y.to_string());
                    log::info!("amount_out_x : {}", &amount_out_x.to_string());
                    log::info!("amount_out_y : {}", &amount_out_y.to_string());

                    let amount_x_traded = get_amount_traded(
                        amount_in_x,
                        amount_out_x,
                        BigInt::from_str(&token_x_decimals).unwrap().to_i32(),
                    );
                    let amount_y_traded = get_amount_traded(
                        amount_in_y,
                        amount_out_y,
                        BigInt::from_str(&token_y_decimals).unwrap().to_i32(),
                    );
                    log::info!("amount_x_traded : {}", &amount_x_traded.to_string());
                    log::info!("amount_y_traded : {}", &amount_y_traded.to_string());

                    let amount_0_traded = get_sorted_amount0(
                        &token_x_address,
                        &token_y_address,
                        &amount_x_traded.to_string(),
                        &amount_y_traded.to_string(),
                    );
                    let amount_1_traded = get_sorted_amount1(
                        &token_x_address,
                        &token_y_address,
                        &amount_x_traded.to_string(),
                        &amount_y_traded.to_string(),
                    );

                    log::info!("amount_0_traded : {}", &amount_0_traded.to_string());
                    log::info!("amount_1_traded : {}", &amount_1_traded.to_string());

                    o.set(
                        0,
                        generate_key("Candle", &append_0x(&candle_id)),
                        &dexcandlesV2::Candle {
                            time: time_params.to_string(),
                            period: candle_period.to_string(),
                            last_block: s.timestamp.to_string(),
                            token0: token0.to_string(),
                            token1: token1.to_string(),
                            token0_amount_traded: amount_0_traded,
                            token1_amount_traded: amount_1_traded,
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
        o.set(0, generate_key("Token", &token_x.address), &token_x);
        o.set(0, generate_key("Token", &token_y.address), &token_y);
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
