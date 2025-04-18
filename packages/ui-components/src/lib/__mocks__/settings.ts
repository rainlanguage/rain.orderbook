import type { Config } from '@rainlanguage/orderbook';
import { writable } from 'svelte/store';

export const mockConfigSource: Config = {
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
} as unknown as Config;

export const mockSettingsStore = writable<Config>(mockConfigSource);
