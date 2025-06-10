import type { Config } from '@rainlanguage/orderbook';

export const mockConfig: Config = {
  orderbook: {
    version: '1',
    networks: {
      mainnet: {
        key: 'mainnet',
        rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
        chainId: 1,
        label: 'Ethereum Mainnet',
        currency: 'ETH',
      },
    },
    subgraphs: {
      mainnet: {
        key: 'mainnet',
        url: 'https://api.thegraph.com/subgraphs/name/mainnet',
      },
    },
    orderbooks: {
      orderbook1: {
        key: 'orderbook1',
        address: '0xOrderbookAddress1',
        network: {
          key: 'mainnet',
          rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
          chainId: 1,
          label: 'Ethereum Mainnet',
          currency: 'ETH',
        },
        subgraph: {
          key: 'mainnet',
          url: 'https://api.thegraph.com/subgraphs/name/mainnet',
        },
        label: 'Orderbook 1',
      },
    },
    deployers: {
      deployer1: {
        key: 'deployer1',
        address: '0xDeployerAddress1',
        network: {
          key: 'mainnet',
          rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
          chainId: 1,
        },
      },
    },
    metaboards: {
      metaboard1: 'https://example.com/metaboard1',
    },
    accounts: {
      name_one: {
        key: 'name_one',
        address: 'address_one',
      },
      name_two: {
        key: 'name_two',
        address: 'address_two',
      },
    },
  },
} as unknown as Config;
