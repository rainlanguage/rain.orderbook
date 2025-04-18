import type {
	AppStoresInterface,
	ConfigSource,
	OrderbookConfigSource,
	OrderbookCfgRef
} from '@rainlanguage/ui-components';
import { writable, derived, get } from 'svelte/store';
import pkg from 'lodash';

const { pickBy } = pkg;

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = async ({ fetch }) => {
	const response = await fetch(
		'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/settings.json'
	);
	const settingsJson = await response.json();

	// const settingsJson = {
	// 	accounts: {},
	// 	networks: {
	// 		flare: {
	// 			key: 'flare',
	// 			rpc: 'https://flare.rpc.thirdweb.com',
	// 			'chain-id': 14,
	// 			currency: 'FLR'
	// 		},
	// 		base: {
	// 			key: 'base',
	// 			rpc: 'https://base-rpc.publicnode.com',
	// 			'chain-id': 8453,
	// 			'network-id': 8453,
	// 			currency: 'ETH'
	// 		},
	// 		polygon: {
	// 			key: 'polygon',
	// 			rpc: 'https://1rpc.io/matic',
	// 			'chain-id': 137,
	// 			'network-id': 137,
	// 			currency: 'POL'
	// 		},
	// 		arbitrum: {
	// 			key: 'arbitrum',
	// 			rpc: 'https://1rpc.io/arb',
	// 			'chain-id': 42161,
	// 			'network-id': 42161,
	// 			currency: 'ETH'
	// 		},
	// 		bsc: {
	// 			key: 'bsc',
	// 			rpc: 'https://bsc-dataseed.bnbchain.org',
	// 			'chain-id': 56,
	// 			'network-id': 56,
	// 			currency: 'BNB'
	// 		},
	// 		linea: {
	// 			key: 'linea',
	// 			rpc: 'https://rpc.linea.build',
	// 			'chain-id': 59144,
	// 			'network-id': 59144,
	// 			currency: 'ETH'
	// 		},
	// 		ethereum: {
	// 			key: 'ethereum',
	// 			rpc: 'https://1rpc.io/eth',
	// 			'chain-id': 1,
	// 			'network-id': 1,
	// 			currency: 'ETH'
	// 		}
	// 	},
	// 	subgraphs: {
	// 		flare: {
	// 			key: 'flare',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/2024-12-13-9dc7/gn'
	// 		},
	// 		base: {
	// 			key: 'base',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/2024-12-13-9c39/gn'
	// 		},
	// 		polygon: {
	// 			key: 'polygon',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-matic/2024-12-13-d2b4/gn'
	// 		},
	// 		arbitrum: {
	// 			key: 'arbitrum',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-arbitrum-one/2024-12-13-7435/gn'
	// 		},
	// 		bsc: {
	// 			key: 'bsc',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-bsc/2024-12-13-2244/gn'
	// 		},
	// 		linea: {
	// 			key: 'linea',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-linea/2024-12-13-09c7/gn'
	// 		},
	// 		ethereum: {
	// 			key: 'ethereum',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-12-13-7f22/gn'
	// 		}
	// 	},
	// 	metaboards: {
	// 		flare: {
	// 			key: 'flare',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn'
	// 		},
	// 		base: {
	// 			key: 'base',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn'
	// 		},
	// 		polygon: {
	// 			key: 'polygon',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-polygon/0.1/gn'
	// 		},
	// 		arbitrum: {
	// 			key: 'arbitrum',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-arbitrum/0.1/gn'
	// 		},
	// 		bsc: {
	// 			key: 'bsc',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-bsc/0.1/gn'
	// 		},
	// 		linea: {
	// 			key: 'linea',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-linea-0xed7d6156/1.0.0/gn'
	// 		},
	// 		ethereum: {
	// 			key: 'ethereum',
	// 			url: 'https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/metadata-mainnet/2024-10-25-2857/gn'
	// 		}
	// 	},
	// 	orderbooks: {
	// 		flare: {
	// 			key: 'flare',
	// 			address: '0xCEe8Cd002F151A536394E564b84076c41bBBcD4d',
	// 			network: 'flare',
	// 			subgraph: 'flare'
	// 		},
	// 		base: {
	// 			key: 'base',
	// 			address: '0xd2938e7c9fe3597f78832ce780feb61945c377d7',
	// 			network: 'base',
	// 			subgraph: 'base'
	// 		},
	// 		polygon: {
	// 			key: 'polygon',
	// 			address: '0x7D2f700b1f6FD75734824EA4578960747bdF269A',
	// 			network: 'polygon',
	// 			subgraph: 'polygon'
	// 		},
	// 		arbitrum: {
	// 			key: 'arbitrum',
	// 			address: '0x550878091b2B1506069F61ae59e3A5484Bca9166',
	// 			network: 'arbitrum',
	// 			subgraph: 'arbitrum'
	// 		},
	// 		matchain: {
	// 			key: 'matchain',
	// 			address: '0x40312edab8fe65091354172ad79e9459f21094e2',
	// 			network: 'matchain',
	// 			subgraph: 'matchain'
	// 		},
	// 		bsc: {
	// 			key: 'bsc',
	// 			address: '0xd2938E7c9fe3597F78832CE780Feb61945c377d7',
	// 			network: 'bsc',
	// 			subgraph: 'bsc'
	// 		},
	// 		linea: {
	// 			key: 'linea',
	// 			address: '0x22410e2a46261a1B1e3899a072f303022801C764',
	// 			network: 'linea',
	// 			subgraph: 'linea'
	// 		},
	// 		ethereum: {
	// 			key: 'ethereum',
	// 			address: '0x0eA6d458488d1cf51695e1D6e4744e6FB715d37C',
	// 			network: 'ethereum',
	// 			subgraph: 'ethereum'
	// 		}
	// 	},
	// 	deployers: {
	// 		flare: {
	// 			key: 'flare',
	// 			address: '0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb',
	// 			network: 'flare'
	// 		},
	// 		base: {
	// 			key: 'base',
	// 			address: '0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D',
	// 			network: 'base'
	// 		},
	// 		polygon: {
	// 			key: 'polygon',
	// 			address: '0xE7116BC05C8afe25e5B54b813A74F916B5D42aB1',
	// 			network: 'polygon'
	// 		},
	// 		arbitrum: {
	// 			key: 'arbitrum',
	// 			address: '0x9B0D254bd858208074De3d2DaF5af11b3D2F377F',
	// 			network: 'arbitrum'
	// 		},
	// 		matchain: {
	// 			key: 'matchain',
	// 			address: '0x582d9e838FE6cD9F8147C66A8f56A3FBE513a6A2',
	// 			network: 'polygon'
	// 		},
	// 		bsc: {
	// 			key: 'bsc',
	// 			address: '0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA',
	// 			network: 'bsc'
	// 		},
	// 		linea: {
	// 			key: 'linea',
	// 			address: '0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA',
	// 			network: 'linea'
	// 		},
	// 		ethereum: {
	// 			key: 'ethereum',
	// 			address: '0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77',
	// 			network: 'ethereum'
	// 		}
	// 	}
	// };

	const settings = writable<ConfigSource | undefined>(settingsJson);
	const activeNetworkRef = writable<string>('');
	const activeOrderbookRef = writable<string>('');
	const activeOrderbook = derived(
		[settings, activeOrderbookRef],
		([$settings, $activeOrderbookRef]) =>
			$settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
				? $settings.orderbooks[$activeOrderbookRef]
				: undefined
	);

	const activeNetworkOrderbooks = derived(
		[settings, activeNetworkRef],
		([$settings, $activeNetworkRef]) =>
			$settings?.orderbooks
				? (pickBy(
						$settings.orderbooks,
						(orderbook) => orderbook.network === $activeNetworkRef
					) as Record<OrderbookCfgRef, OrderbookConfigSource>)
				: ({} as Record<OrderbookCfgRef, OrderbookConfigSource>)
	);

	const accounts = derived(settings, ($settings) => $settings?.accounts);
	const activeAccountsItems = writable<Record<string, string>>({});

	const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
		$settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
			? $settings.subgraphs[$activeOrderbook.subgraph]
			: undefined
	);
	const activeAccounts = derived(
		[accounts, activeAccountsItems],
		([$accounts, $activeAccountsItems]) =>
			Object.keys($activeAccountsItems).length === 0
				? {}
				: Object.fromEntries(
						Object.entries($accounts || {}).filter(([key]) => key in $activeAccountsItems)
					)
	);

	return {
		stores: {
			settings,
			activeSubgraphs: writable<Record<string, string>>({}),
			accounts,
			activeAccountsItems,
			activeAccounts,
			activeOrderStatus: writable<boolean | undefined>(undefined),
			orderHash: writable<string>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			activeNetworkRef,
			activeOrderbookRef,
			activeOrderbook,
			subgraphUrl,
			activeNetworkOrderbooks
		}
	};
};

export const ssr = false;

if (import.meta.vitest) {
	const { describe, it, expect, beforeEach, vi } = import.meta.vitest;

	const mockFetch = vi.fn();
	vi.stubGlobal('fetch', mockFetch);

	describe('Layout load function', () => {
		const mockSettingsJson = {
			accounts: {
				account1: { name: 'Test Account 1' },
				account2: { name: 'Test Account 2' }
			},
			orderbooks: {
				orderbook1: {
					key: 'orderbook1',
					network: 'network1',
					subgraph: 'subgraph1'
				},
				orderbook2: {
					key: 'orderbook2',
					network: 'network2',
					subgraph: 'subgraph2'
				},
				orderbook3: {
					key: 'orderbook3',
					network: 'network1',
					subgraph: 'subgraph3'
				}
			},
			subgraphs: {
				subgraph1: {
					key: 'subgraph1',
					url: 'https://subgraph1.url'
				},
				subgraph2: {
					key: 'subgraph2',
					url: 'https://subgraph2.url'
				},
				subgraph3: {
					key: 'subgraph3',
					url: 'https://subgraph3.url'
				}
			}
		};

		beforeEach(() => {
			vi.clearAllMocks();
			vi.resetAllMocks();
		});

		it('should load settings and initialize stores correctly', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(mockFetch).toHaveBeenCalledWith(
				'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/settings.json'
			);

			expect(result).toHaveProperty('stores');
			const { stores } = result;

			expect(stores).toHaveProperty('settings');
			expect(stores).toHaveProperty('activeSubgraphs');
			expect(stores).toHaveProperty('accounts');
			expect(stores).toHaveProperty('activeAccountsItems');
			expect(stores).toHaveProperty('activeAccounts');
			expect(stores).toHaveProperty('activeOrderStatus');
			expect(stores).toHaveProperty('orderHash');
			expect(stores).toHaveProperty('hideZeroBalanceVaults');
			expect(stores).toHaveProperty('activeNetworkRef');
			expect(stores).toHaveProperty('activeOrderbookRef');
			expect(stores).toHaveProperty('activeOrderbook');
			expect(stores).toHaveProperty('subgraphUrl');
			expect(stores).toHaveProperty('activeNetworkOrderbooks');

			expect(get(stores.settings)).toEqual(mockSettingsJson);
			expect(get(stores.activeNetworkRef)).toEqual('');
			expect(get(stores.activeOrderbookRef)).toEqual('');
			expect(get(stores.activeAccountsItems)).toEqual({});
			expect(get(stores.orderHash)).toEqual('');
			expect(get(stores.hideZeroBalanceVaults)).toEqual(false);
		});

		it('should handle derived store: activeOrderbook', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeOrderbook)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);
		});

		it('should handle derived store: activeNetworkOrderbooks', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeNetworkRef.set('network1');

			const networkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(networkOrderbooks).toHaveProperty('orderbook1');
			expect(networkOrderbooks).toHaveProperty('orderbook3');
			expect(networkOrderbooks).not.toHaveProperty('orderbook2');
		});

		it('should handle derived store: subgraphUrl', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.subgraphUrl)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.subgraphUrl)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });
		});

		it('should handle derived store: activeAccounts with empty activeAccountsItems', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeAccounts)).toEqual({});
		});

		it('should handle derived store: activeAccounts with filled activeAccountsItems', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			stores.activeAccountsItems.set({ account1: 'Account 1' });

			const accounts = get(stores.activeAccounts);
			expect(accounts).toHaveProperty('account1');
			expect(accounts).not.toHaveProperty('account2');
		});

		it('should handle fetch failure', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			await expect(load({ fetch: mockFetch } as any)).rejects.toThrow('Network error');
		});

		it('should handle empty or malformed settings JSON', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve({})
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.settings)).toEqual({});
			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraphUrl)).toBeUndefined();
		});

		it('should handle chain reaction of store updates when changing network and orderbook', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraphUrl)).toBeUndefined();
			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeNetworkRef.set('network1');

			const networkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(networkOrderbooks).length).toBe(2);
			expect(networkOrderbooks).toHaveProperty('orderbook1');
			expect(networkOrderbooks).toHaveProperty('orderbook3');

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraphUrl)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);
			expect(get(stores.subgraphUrl)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });

			stores.activeNetworkRef.set('network2');

			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);

			const newNetworkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(newNetworkOrderbooks).length).toBe(1);
			expect(newNetworkOrderbooks).toHaveProperty('orderbook2');
			expect(newNetworkOrderbooks).not.toHaveProperty('orderbook1');
		});

		it('should handle multiple interrelated store updates correctly', async () => {
			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			stores.activeNetworkRef.set('network1');
			stores.activeOrderbookRef.set('orderbook1');
			stores.activeAccountsItems.set({ account1: 'Account 1' });

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);
			expect(get(stores.subgraphUrl)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });
			expect(get(stores.activeAccounts)).toHaveProperty('account1');

			stores.activeNetworkRef.set('network2');

			stores.activeAccountsItems.set({ account1: 'Account 1', account2: 'Account 2' });

			stores.activeOrderbookRef.set('orderbook2');

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook2');
			expect(get(stores.activeNetworkOrderbooks)).not.toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook2);
			expect(get(stores.subgraphUrl)).toEqual({ key: 'subgraph2', url: 'https://subgraph2.url' });

			const finalAccounts = get(stores.activeAccounts);
			expect(Object.keys(finalAccounts).length).toBe(2);
			expect(finalAccounts).toHaveProperty('account1');
			expect(finalAccounts).toHaveProperty('account2');
		});

		it('should handle partial or invalid data in settings correctly', async () => {
			const partialSettings = {
				accounts: mockSettingsJson.accounts,
				orderbooks: {
					orderbook1: {
						key: 'orderbook1',
						network: 'network1'
					},
					orderbook2: {
						key: 'orderbook2',
						network: 'network2',
						subgraph: 'nonexistent_subgraph'
					}
				}
			};

			mockFetch.mockResolvedValueOnce({
				json: () => Promise.resolve(partialSettings)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			stores.activeOrderbookRef.set('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(partialSettings.orderbooks.orderbook1);
			expect(get(stores.subgraphUrl)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook2');
			expect(get(stores.activeOrderbook)).toEqual(partialSettings.orderbooks.orderbook2);
			expect(get(stores.subgraphUrl)).toBeUndefined();
		});
	});
}
