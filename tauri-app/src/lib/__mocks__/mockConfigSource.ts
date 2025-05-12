import type { ConfigSource } from '@rainlanguage/orderbook';

export const mockConfigSource: ConfigSource = {
  networks: {
    mainnet: {
      rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
      'chain-id': 1,
      label: 'Ethereum Mainnet',
      currency: 'ETH',
    },
  },
  subgraphs: {
    mainnet: 'https://api.thegraph.com/subgraphs/name/mainnet',
  },
  orderbooks: {
    orderbook1: {
      address: '0xOrderbookAddress1',
      network: 'mainnet',
      subgraph: 'uniswap',
      label: 'Orderbook 1',
    },
  },
  deployers: {
    deployer1: {
      address: '0xDeployerAddress1',
      network: 'mainnet',
      label: 'Deployer 1',
    },
  },
  metaboards: {
    metaboard1: 'https://example.com/metaboard1',
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
};
