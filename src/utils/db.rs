use crate::pb::traderjoe::dexcandlesv2 as dexcandlesV2;

use std::str::FromStr;
use substreams::scalar::BigInt;

use substreams_entity_change::pb::entity::EntityChanges;

use substreams::store::{DeltaProto, Deltas};
use substreams_entity_change::tables::Tables;

//CREATE
pub fn create_candle_entity(tables: &mut Tables, deltas: Deltas<DeltaProto<dexcandlesV2::Candle>>) {
}
