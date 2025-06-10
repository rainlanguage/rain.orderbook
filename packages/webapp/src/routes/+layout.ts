import type { AppStoresInterface } from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import pickBy from 'lodash/pickBy';
import type { LayoutLoad } from './$types';
import {
	parseYaml,
	type NewConfig,
	type OrderbookCfg,
	type SubgraphCfg
} from '@rainlanguage/orderbook';

export interface LayoutData {
	errorMessage?: string;
	stores: AppStoresInterface | null;
}

export const load: LayoutLoad<LayoutData> = async ({ fetch }) => {
	let config: NewConfig;
	let errorMessage: string | undefined;

	try {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.strategies/037747012240788495b8341ee40dfd407d1fedd3/settings.yaml'
		);
		if (!response.ok) {
			throw new Error('Error status: ' + response.status.toString());
		}
		const settingsYamlText = await response.text();

		const configRes = parseYaml([settingsYamlText]);
		if (configRes.error) {
			return {
				errorMessage: configRes.error.readableMsg,
				stores: null
			};
		}
		config = configRes.value;
	} catch (error: unknown) {
		errorMessage = 'Failed to get site config settings. ' + (error as Error).message;
		return {
			errorMessage,
			stores: null
		};
	}

	const settings: AppStoresInterface['settings'] = writable<NewConfig>(config);
	const activeNetworkRef: AppStoresInterface['activeNetworkRef'] = writable<string>('');
	const activeOrderbookRef: AppStoresInterface['activeOrderbookRef'] = writable<string>('');
	const activeOrderbook = derived(
		[settings, activeOrderbookRef],
		([$settings, $activeOrderbookRef]) =>
			$settings.orderbook.orderbooks !== undefined &&
			Object.entries($settings.orderbook.orderbooks).length > 0 &&
			$activeOrderbookRef !== undefined
				? $settings.orderbook.orderbooks[$activeOrderbookRef]
				: undefined
	);

	const activeNetworkOrderbooks = derived(
		[settings, activeNetworkRef],
		([$settings, $activeNetworkRef]) => {
			return $settings.orderbook.orderbooks
				? (pickBy(
						$settings.orderbook.orderbooks,
						(orderbook) => orderbook.network.key === $activeNetworkRef
					) as Record<string, OrderbookCfg>)
				: ({} as Record<string, OrderbookCfg>);
		}
	);
	const accounts = derived(settings, ($settings) => $settings.orderbook.accounts || {});
	const activeAccountsItems = writable<Record<string, string>>({});

	const subgraph = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
		$settings.orderbook.subgraphs !== undefined &&
		Object.entries($settings.orderbook.subgraphs).length > 0 &&
		$activeOrderbook?.subgraph !== undefined
			? $settings.orderbook.subgraphs[$activeOrderbook.subgraph.key]
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
			activeSubgraphs: writable<Record<string, SubgraphCfg>>({}),
			accounts,
			activeAccountsItems,
			activeAccounts,
			// Instantiate with false to show only active orders
			showInactiveOrders: writable<boolean>(false),
			orderHash: writable<string>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			activeNetworkRef,
			activeOrderbookRef,
			activeOrderbook,
			subgraph,
			activeNetworkOrderbooks,
			showMyItemsOnly: writable<boolean>(false)
		}
	};
};

export const ssr = false;

if (import.meta.vitest) {
	const { describe, it, expect, beforeEach, vi } = import.meta.vitest;
	const { get } = await import('svelte/store');

	const mockFetch = vi.fn();
	vi.stubGlobal('fetch', mockFetch);

	vi.mock('@rainlanguage/orderbook', async (importOriginal) => {
		return {
			...(await importOriginal()),
			parseYaml: vi.fn()
		};
	});

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
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(mockFetch).toHaveBeenCalledWith(
				'https://raw.githubusercontent.com/rainlanguage/rain.strategies/037747012240788495b8341ee40dfd407d1fedd3/settings.yaml'
			);

			expect(result).toHaveProperty('stores');
			const stores: AppStoresInterface | null = result.stores;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(stores).toHaveProperty('settings');
			expect(stores).toHaveProperty('activeSubgraphs');
			expect(stores).toHaveProperty('accounts');
			expect(stores).toHaveProperty('activeAccountsItems');
			expect(stores).toHaveProperty('activeAccounts');
			expect(stores).toHaveProperty('showInactiveOrders');
			expect(stores).toHaveProperty('orderHash');
			expect(stores).toHaveProperty('hideZeroBalanceVaults');
			expect(stores).toHaveProperty('activeNetworkRef');
			expect(stores).toHaveProperty('activeOrderbookRef');
			expect(stores).toHaveProperty('activeOrderbook');
			expect(stores).toHaveProperty('subgraph');
			expect(stores).toHaveProperty('activeNetworkOrderbooks');

			expect(get(stores.settings)).toEqual(mockConfig);
			expect(get(stores.activeNetworkRef)).toEqual('');
			expect(get(stores.activeOrderbookRef)).toEqual('');
			if (stores.activeAccountsItems) {
				expect(get(stores.activeAccountsItems)).toEqual({});
			}
			expect(get(stores.orderHash)).toEqual('');
			expect(get(stores.hideZeroBalanceVaults)).toEqual(false);
		});

		it('should handle derived store: activeOrderbook', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.activeOrderbook)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);
		});

		it('should handle derived store: activeNetworkOrderbooks', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeNetworkRef.set('network1');

			const networkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(networkOrderbooks).toHaveProperty('orderbook1');
			expect(networkOrderbooks).toHaveProperty('orderbook3');
			expect(networkOrderbooks).not.toHaveProperty('orderbook2');
		});

		it('should handle derived store: subgraph', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.subgraph)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.subgraph)).toEqual(mockConfig.orderbook.subgraphs.subgraph1);
		});

		it('should handle derived store: activeAccounts with empty activeAccountsItems', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.activeAccounts)).toEqual({});
		});

		it('should handle derived store: activeAccounts with filled activeAccountsItems', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			stores.activeAccountsItems?.set({ account1: 'Account 1' });

			const accounts = get(stores.activeAccounts);
			expect(accounts).toHaveProperty('account1');
			expect(accounts).not.toHaveProperty('account2');
		});

		it('should return errorMessage if fetch fails with non-OK status', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 404,
				statusText: 'Not Found'
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Error status: 404');
		});

		it('should return errorMessage if response.json() fails', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.reject(new Error('Invalid JSON'))
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Invalid JSON');
		});

		it('should handle fetch failure', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Failed to get site config settings.');
			expect(result.errorMessage).toContain('Network error');
		});

		it('should handle empty or malformed settings JSON', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: undefined,
				error: {
					msg: 'Malformed settings',
					readableMsg: 'Malformed settings'
				}
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve('malformed')
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(result).toHaveProperty('stores', null);
			expect(result).toHaveProperty('errorMessage');
			expect(result.errorMessage).toContain('Malformed settings');
			expect(result.errorMessage).toContain('Malformed settings');
		});

		it('should handle chain reaction of store updates when changing network and orderbook', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

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
			expect(get(stores.subgraph)).toEqual(mockConfig.orderbook.subgraphs.subgraph1);

			stores.activeNetworkRef.set('network2');

			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);

			const newNetworkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(newNetworkOrderbooks).length).toBe(1);
			expect(newNetworkOrderbooks).toHaveProperty('orderbook2');
			expect(newNetworkOrderbooks).not.toHaveProperty('orderbook1');
		});

		it('should handle multiple interrelated store updates correctly', async () => {
			vi.mocked(parseYaml).mockReturnValue({
				value: mockConfig as unknown as NewConfig,
				error: undefined
			});
			mockFetch.mockResolvedValueOnce({
				ok: true,
				text: () => Promise.resolve(mockSettingsYaml)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			stores.activeNetworkRef.set('network1');
			stores.activeOrderbookRef.set('orderbook1');
			stores.activeAccountsItems?.set({ account1: 'Account 1' });

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook1);
			expect(get(stores.subgraph)).toEqual(mockConfig.orderbook.subgraphs.subgraph1);
			expect(get(stores.activeAccounts)).toHaveProperty('account1');

			stores.activeNetworkRef.set('network2');
			stores.activeAccountsItems?.set({ account1: 'Account 1', account2: 'Account 2' });
			stores.activeOrderbookRef.set('orderbook2');

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook2');
			expect(get(stores.activeNetworkOrderbooks)).not.toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockConfig.orderbook.orderbooks.orderbook2);
			expect(get(stores.subgraph)).toEqual(mockConfig.orderbook.subgraphs.subgraph2);

			const finalAccounts = get(stores.activeAccounts);
			expect(Object.keys(finalAccounts).length).toBe(2);
			expect(finalAccounts).toHaveProperty('account1');
			expect(finalAccounts).toHaveProperty('account2');
		});
	});
}
