import type {
	AppStoresInterface,
	ConfigSource,
	OrderbookConfigSource,
	OrderbookCfgRef
} from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import pickBy from 'lodash/pickBy';
import type { LayoutLoad } from './$types';

export interface LayoutData {
	errorMessage?: string;
	stores: AppStoresInterface | null;
}

export const load: LayoutLoad<LayoutData> = async ({ fetch }) => {
	let settingsJson: ConfigSource | undefined;
	let errorMessage: string | undefined;

	try {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/settings.json'
		);
		if (!response.ok) {
			throw new Error('Error status: ' + response.status.toString());
		}
		settingsJson = await response.json();
	} catch (error: unknown) {
		errorMessage = 'Failed to get site config settings. ' + (error as Error).message;
		return {
			errorMessage,
			stores: null
		};
	}

	const settings: AppStoresInterface['settings'] = writable<ConfigSource | undefined>(settingsJson);
	const activeNetworkRef: AppStoresInterface['activeNetworkRef'] = writable<string>('');
	const activeOrderbookRef: AppStoresInterface['activeOrderbookRef'] = writable<string>('');
	const activeOrderbook: AppStoresInterface['activeOrderbook'] = derived(
		[settings, activeOrderbookRef],
		([$settings, $activeOrderbookRef]) =>
			$settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
				? $settings.orderbooks[$activeOrderbookRef]
				: undefined
	);

	const activeNetworkOrderbooks: AppStoresInterface['activeNetworkOrderbooks'] = derived(
		[settings, activeNetworkRef],
		([$settings, $activeNetworkRef]) =>
			$settings?.orderbooks
				? (pickBy(
						$settings.orderbooks,
						(orderbook) => orderbook.network === $activeNetworkRef
					) as Record<OrderbookCfgRef, OrderbookConfigSource>)
				: ({} as Record<OrderbookCfgRef, OrderbookConfigSource>)
	);

	const accounts: AppStoresInterface['accounts'] = derived(
		settings,
		($settings) => $settings?.accounts ?? {}
	);
	const activeAccountsItems: AppStoresInterface['activeAccountsItems'] = writable<
		Record<string, string>
	>({});

	const subgraphUrl: AppStoresInterface['subgraphUrl'] = derived(
		[settings, activeOrderbook],
		([$settings, $activeOrderbook]) =>
			$settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
				? $settings.subgraphs[$activeOrderbook.subgraph]
				: undefined
	);
	const activeAccounts: AppStoresInterface['activeAccounts'] = derived(
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
			// Instantiate with false to show only active orders
			showInactiveOrders: writable<boolean>(false),
			orderHash: writable<string>(''),
			hideZeroBalanceVaults: writable<boolean>(false),
			activeNetworkRef,
			activeOrderbookRef,
			activeOrderbook,
			subgraphUrl,
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

	describe('Layout load function', () => {
		const mockSettingsJson = {
			accounts: {
				account1: { name: 'Test Account 1' },
				account2: { name: 'Test Account 2' }
			},
			orderbooks: {
				orderbook1: {
					network: 'network1',
					subgraph: 'subgraph1'
				},
				orderbook2: {
					network: 'network2',
					subgraph: 'subgraph2'
				},
				orderbook3: {
					network: 'network1',
					subgraph: 'subgraph3'
				}
			},
			subgraphs: {
				subgraph1: 'https://subgraph1.url',
				subgraph2: 'https://subgraph2.url',
				subgraph3: 'https://subgraph3.url'
			}
		};

		beforeEach(() => {
			vi.clearAllMocks();
			vi.resetAllMocks();
		});

		it('should load settings and initialize stores correctly', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);

			expect(mockFetch).toHaveBeenCalledWith(
				'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/settings.json'
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
			expect(stores).toHaveProperty('subgraphUrl');
			expect(stores).toHaveProperty('activeNetworkOrderbooks');

			expect(get(stores.settings)).toEqual(mockSettingsJson);
			expect(get(stores.activeNetworkRef)).toEqual('');
			expect(get(stores.activeOrderbookRef)).toEqual('');
			if (stores.activeAccountsItems) {
				expect(get(stores.activeAccountsItems)).toEqual({});
			}
			expect(get(stores.orderHash)).toEqual('');
			expect(get(stores.hideZeroBalanceVaults)).toEqual(false);
		});

		it('should handle derived store: activeOrderbook', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.activeOrderbook)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);
		});

		it('should handle derived store: activeNetworkOrderbooks', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
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

		it('should handle derived store: subgraphUrl', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.subgraphUrl)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.subgraphUrl)).toEqual('https://subgraph1.url');
		});

		it('should handle derived store: activeAccounts with empty activeAccountsItems', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.activeAccounts)).toEqual({});
		});

		it('should handle derived store: activeAccounts with filled activeAccountsItems', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
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
				json: () => Promise.reject(new Error('Invalid JSON'))
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
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({})
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			expect(get(stores.settings)).toEqual({});
			expect(get(stores.activeNetworkOrderbooks)).toEqual({});

			stores.activeOrderbookRef.set('orderbook1');

			expect(get(stores.activeOrderbook)).toBeUndefined();
			expect(get(stores.subgraphUrl)).toBeUndefined();
		});

		it('should handle chain reaction of store updates when changing network and orderbook', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

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
			expect(get(stores.subgraphUrl)).toEqual('https://subgraph1.url');

			stores.activeNetworkRef.set('network2');

			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);

			const newNetworkOrderbooks = get(stores.activeNetworkOrderbooks);
			expect(Object.keys(newNetworkOrderbooks).length).toBe(1);
			expect(newNetworkOrderbooks).toHaveProperty('orderbook2');
			expect(newNetworkOrderbooks).not.toHaveProperty('orderbook1');
		});

		it('should handle multiple interrelated store updates correctly', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(mockSettingsJson)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			stores.activeNetworkRef.set('network1');
			stores.activeOrderbookRef.set('orderbook1');
			stores.activeAccountsItems?.set({ account1: 'Account 1' });

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook1);
			expect(get(stores.subgraphUrl)).toEqual('https://subgraph1.url');
			expect(get(stores.activeAccounts)).toHaveProperty('account1');

			stores.activeNetworkRef.set('network2');
			stores.activeAccountsItems?.set({ account1: 'Account 1', account2: 'Account 2' });
			stores.activeOrderbookRef.set('orderbook2');

			expect(get(stores.activeNetworkOrderbooks)).toHaveProperty('orderbook2');
			expect(get(stores.activeNetworkOrderbooks)).not.toHaveProperty('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(mockSettingsJson.orderbooks.orderbook2);
			expect(get(stores.subgraphUrl)).toEqual('https://subgraph2.url');

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
						network: 'network1'
					},
					orderbook2: {
						network: 'network2',
						subgraph: 'nonexistent_subgraph'
					}
				}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve(partialSettings)
			});

			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			const result = await load({ fetch: mockFetch } as any);
			const { stores } = result;

			if (!stores) throw new Error('Test setup error: stores should not be null');

			stores.activeOrderbookRef.set('orderbook1');
			expect(get(stores.activeOrderbook)).toEqual(partialSettings.orderbooks.orderbook1);
			expect(get(stores.subgraphUrl)).toBeUndefined();

			stores.activeOrderbookRef.set('orderbook2');
			expect(get(stores.activeOrderbook)).toEqual(partialSettings.orderbooks.orderbook2);
			expect(get(stores.subgraphUrl)).toBeUndefined();
		});
	});
}
