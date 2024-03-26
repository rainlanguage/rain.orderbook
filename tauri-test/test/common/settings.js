const coreSettings = `
networks: 
  polygon: 
    rpc: https://rpc.ankr.com/polygon 
    chain-id: 137 
    label: Polygon 
    network-id: 137 
    currency: MATIC 

subgraphs:
  polygon: https://api.thegraph.com/subgraphs/name/siddharth2207/obv3subparser

orderbooks:
  polygonOB:
    address: 0xDE5aBE2837bc042397D80E37fb7b2C850a8d5a6C
    network: polygon
    subgraph: polygon
    label: Polygon Orderbook
`;

module.exports = {
  coreSettings
};