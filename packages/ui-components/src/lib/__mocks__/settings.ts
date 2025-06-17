import type { NewConfig } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';

export const mockConfig: NewConfig = {
	orderbook: {
		networks: {
			mainnet: {
				key: 'mainnet',
				rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
				chainId: 1,
				label: 'Ethereum Mainnet',
				currency: 'ETH'
			}
		},
		subgraphs: {
			mainnet: {
				key: 'mainnet',
				url: 'https://api.thegraph.com/subgraphs/name/mainnet'
			},
			flare: {
				key: 'flare',
				url: 'https://api.thegraph.com/subgraphs/name/flare'
			},
			testnet: {
				key: 'testnet',
				url: 'https://api.thegraph.com/subgraphs/name/testnet'
			}
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
					currency: 'ETH'
				},
				subgraph: {
					key: 'uniswap',
					url: 'https://api.thegraph.com/subgraphs/name/uniswap'
				},
				label: 'Orderbook 1'
			}
		},
		deployers: {
			deployer1: {
				key: 'deployer1',
				address: '0xDeployerAddress1',
				network: {
					key: 'mainnet',
					rpc: 'https://mainnet.infura.io/v3/YOUR-PROJECT-ID',
					chainId: 1,
					label: 'Ethereum Mainnet',
					currency: 'ETH'
				}
			}
		},
		metaboards: {
			metaboard1: 'https://example.com/metaboard1'
		},
		accounts: {
			name_one: 'address_one',
			name_two: 'address_two'
		}
	}
} as unknown as NewConfig;

const mockSettingsStoreWritable = writable<NewConfig>(mockConfig);

export const mockSettingsStore = {
	subscribe: mockSettingsStoreWritable.subscribe,
	set: mockSettingsStoreWritable.set,
	mockSetSubscribeValue: (value: NewConfig) => mockSettingsStoreWritable.set(value)
};
