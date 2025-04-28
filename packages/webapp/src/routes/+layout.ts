import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { parseYaml } from '@rainlanguage/orderbook';
import type { Config, OrderbookCfg, OrderbookConfig } from '@rainlanguage/orderbook';
import { writable, derived, get } from 'svelte/store';
import pickBy from 'lodash/pickBy';

export interface LayoutData {
	stores: AppStoresInterface;
}

export const load = async ({ fetch }) => {
	const response = await fetch(
		'https://raw.githubusercontent.com/findolor/sample-dotrains/6865d8c76dc4182c5f6f41c20299cb017a6298fe/settings.yaml'
	);
	const settingsYaml = await response.text();

	const settings = writable<OrderbookConfig>(parseYaml([settingsYaml]).orderbook);
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
		([$settings, $activeNetworkRef]) => {
			return $settings?.orderbooks
				? (pickBy(
						$settings.orderbooks as unknown as Record<string, OrderbookCfg>,
						(orderbook) => orderbook.network.key === $activeNetworkRef
					) as Record<string, OrderbookCfg>)
				: ({} as Record<string, OrderbookCfg>);
		}
	);

	const accounts = derived(settings, ($settings) => $settings?.accounts);
	const activeAccountsItems = writable<Record<string, string>>({});

	const subgraph = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
		$settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
			? $settings.subgraphs[$activeOrderbook.subgraph.key]
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
			subgraph,
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
		const mockSettingsYaml = `
accounts:
  account1: 0x1234567890123456789012345678901234567890
  account2: 0x1234567890123456789012345678901234567890
networks:
  network1:
    rpc: https://network1.rpc
    chainId: 1
    label: Network 1
    currency: ETH
  network2:
    rpc: https://network2.rpc
    chainId: 2
    label: Network 2
    currency: ETH
orderbooks:
  orderbook1:
    address: 0x1234567890123456789012345678901234567890
    network: network1
    subgraph: subgraph1
  orderbook2:
    address: 0x1234567890123456789012345678901234567890
    network: network2
    subgraph: subgraph2
  orderbook3:
    address: 0x1234567890123456789012345678901234567890
    network: network1
    subgraph: subgraph3
subgraphs:
  subgraph1: https://subgraph1.url
  subgraph2: https://subgraph2.url
  subgraph3: https://subgraph3.url
`;
		const network1 = {
			key: 'network1',
			rpc: 'https://network1.rpc',
			chainId: 1,
			label: 'Network 1',
			currency: 'ETH'
		};
		const network2 = {
			key: 'network2',
			rpc: 'https://network2.rpc',
			chainId: 2,
			label: 'Network 2',
			currency: 'ETH'
		};
		const subgraph1 = {
			key: 'subgraph1',
			url: 'https://subgraph1.url'
		};
		const subgraph2 = {
			key: 'subgraph2',
			url: 'https://subgraph2.url'
		};
		const subgraph3 = {
			key: 'subgraph3',
			url: 'https://subgraph3.url'
		};
		const mockConfig = {
			orderbook: {
				accounts: {
					account1: {
						name: 'Test Account 1'
					},
					account2: {
						name: 'Test Account 2'
					}
				},
				networks: {
					network1,
					network2
				},
				subgraphs: {
					subgraph1,
					subgraph2,
					subgraph3
				},
				orderbooks: {
					orderbook1: {
						key: 'orderbook1',
						address: '0x1234567890123456789012345678901234567890',
						network: network1,
						subgraph: subgraph1
					},
					orderbook2: {
						key: 'orderbook2',
						address: '0x1234567890123456789012345678901234567890',
						network: network2,
						subgraph: subgraph2
					},
					orderbook3: {
						key: 'orderbook3',
						address: '0x1234567890123456789012345678901234567890',
						network: network1,
						subgraph: subgraph3
					}
				}
			}
		};

		beforeEach(() => {
			vi.clearAllMocks();
			vi.resetAllMocks();
		});

		it('should load settings and initialize stores correctly', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(mockFetch).toHaveBeenCalledWith(
				'https://raw.githubusercontent.com/findolor/sample-dotrains/6865d8c76dc4182c5f6f41c20299cb017a6298fe/settings.yaml'
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
			expect(stores).toHaveProperty('subgraph');
			expect(stores).toHaveProperty('activeNetworkOrderbooks');

			expect(get(stores.settings)).toEqual(mockConfig.orderbook);
			expect(get(stores.activeNetworkRef)).toEqual('');
			expect(get(stores.activeOrderbookRef)).toEqual('');
			expect(get(stores.activeAccountsItems)).toEqual({});
			expect(get(stores.orderHash)).toEqual('');
			expect(get(stores.hideZeroBalanceVaults)).toEqual(false);
		});

		it('should handle derived store: activeOrderbook', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeOrderbook)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);
		});

		it('should handle derived store: activeNetworkOrderbooks', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
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
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.subgraph)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.subgraph)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });
		});

		it('should handle derived store: activeAccounts with empty activeAccountsItems', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeAccounts)).toEqual({});
		});

		it('should handle derived store: activeAccounts with filled activeAccountsItems', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
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
			vi.mocked(parseYaml).mockReturnValue({ orderbook: {} } as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve({})
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.settings)).toEqual({});
			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraph)).toBeUndefined();
		});

		it('should handle chain reaction of store updates when changing network and orderbook', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraph)).toBeUndefined();
			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeNetworkRef.set('network1');

			const networkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(networkOrderbooks).length).toBe(2);
			expect(networkOrderbooks).toHaveProperty('orderbook1');
			expect(networkOrderbooks).toHaveProperty('orderbook3');

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraph)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);
			expect(get(stores.subgraph)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });

			stores.activeNetworkRef.set('network2');

			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);

			const newNetworkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(newNetworkOrderbooks).length).toBe(1);
			expect(newNetworkOrderbooks).toHaveProperty('orderbook2');
			expect(newNetworkOrderbooks).not.toHaveProperty('orderbook1');
		});

		it('should handle multiple interrelated store updates correctly', async () => {
			vi.mocked(parseYaml).mockReturnValue(mockConfig as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			stores.activeNetworkRef.set('network1');
			stores.activeOrderbookRef.set('orderbook1');
			stores.activeAccountsItems.set({ account1: 'Account 1' });

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);
			expect(get(stores.subgraph)).toEqual({ key: 'subgraph1', url: 'https://subgraph1.url' });
			expect(get(stores.activeAccounts)).toHaveProperty('account1');

			stores.activeNetworkRef.set('network2');

			stores.activeAccountsItems.set({ account1: 'Account 1', account2: 'Account 2' });

			stores.activeOrderbookRef.set('orderbook2');

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook2');
			expect(get(stores.activeNetworkOrderbooks)).not.toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook2);
			expect(get(stores.subgraph)).toEqual({ key: 'subgraph2', url: 'https://subgraph2.url' });

			const finalAccounts = get(stores.activeAccounts);
			expect(Object.keys(finalAccounts).length).toBe(2);
			expect(finalAccounts).toHaveProperty('account1');
			expect(finalAccounts).toHaveProperty('account2');
		});

		it('should handle partial or invalid data in settings correctly', async () => {
			const partialSettings = `
networks:
  network1:
    rpc: https://network1.rpc
    chainId: 1
    label: Network 1
    currency: ETH
subgraphs:
  subgraph1:
    key: subgraph1
    url: https://subgraph1.url
orderbooks:
  orderbook1:
  	address: 0x1234567890123456789012345678901234567890
    network: network1
    subgraph: non-existent-subgraph
  orderbook2:
    address: 0x1234567890123456789012345678901234567890
    network: network1
`;

			vi.mocked(parseYaml).mockReturnValue(partialSettings as unknown as Config);
			mockFetch.mockResolvedValueOnce({
				text: () => Promise.resolve(partialSettings)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			stores.activeOrderbookRef.set('orderbook1');
			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraph)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook2');
			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraph)).toBeUndefined();
		});
	});
}
