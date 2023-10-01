# TraderJoe - DexCandles V2 Substreams

Candle-sticks data for trading on Joe-V2 (5m/15m/1h/4h/1d/1w) powered by Substreams.

The subgraph indexes all trades for a given tokenX-tokenY pair.

## Data Flow
```mermaid
graph TD;
  map_pairs_created[map: map_pairs_created];
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_pairs_created;
  store_pairs[store: store_pairs];
  map_pairs_created --> store_pairs;
  map_swaps[map: map_swaps];
  sf.ethereum.type.v2.Block[source: sf.ethereum.type.v2.Block] --> map_swaps;
  store_tokens[store: store_tokens];
  map_pairs_created --> store_tokens;
  store_candles[store: store_candles];
  store_pairs --> store_candles;
  map_swaps --> store_candles;
  graph_out[map: graph_out];
  store_pairs -- deltas --> graph_out;
  store_tokens -- deltas --> graph_out;
  store_candles -- deltas --> graph_out;

```