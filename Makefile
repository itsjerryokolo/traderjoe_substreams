ENDPOINT ?= mainnet.eth.streamingfast.io:443

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_pairs_created -s 17821282 -t +2000 --debug-modules-output -o json

.PHONY: mpc
mpc: 
	substreams gui -e $(ENDPOINT) substreams.yaml map_pairs_created -s 17821282 -t +50000

PHONY: swap
swap: 
	substreams gui -e $(ENDPOINT) substreams.yaml map_swaps -s 17821282 -t +50000

PHONY: token
token: 
	substreams gui -e $(ENDPOINT) substreams.yaml store_tokens -s 17821282 -t +50000

PHONY: ca
ca: 
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml graph_out -s 17835000 -t +10000 --debug-modules-output=store_candles -o json

PHONY:pa
pa: 
	substreams run -e mainnet.eth.streamingfast.io:443 substreams.yaml graph_out -s 17821282 -t +10000 --debug-modules-output=store_pairs -o json

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: package
package:
	substreams pack ./substreams.yaml
