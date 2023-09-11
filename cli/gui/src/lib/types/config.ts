export type OrderbookComponentsConfig = {
    /**
     * The address of the orderbook contract
     * @example "0x000 ..."
     * @pattern "^0x[a-fA-F0-9]{40}$"
     * @minLength 42
     * @maxLength 42
     * 
     **/
    address: string;
    /**
     * The endpoint of the subgraph
     * @example "https://api.thegraph.com/subgraphs/name/rainprotocol/orderbook-mumbai"
     * @pattern "^https?://.*"
     **/
    subgraphEndpoint: string;
}