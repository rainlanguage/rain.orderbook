import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import VaultsPage from './+page.svelte';
import { useAccount } from '@rainlanguage/ui-components';

const { mockPageStore, initialPageState } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const mockAccountStore = writable('0xabcdef1234567890abcdef1234567890abcdef12');


vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	const original = (await importOriginal()) as object;
	return {
		...original,
		VaultsListTable: MockComponent,
		useAccount: vi.fn()
	};
});

// Mock the page store
const mockShowMyItemsOnly = writable(false);
const mockPageData = {
	data: {
		stores: {
			activeOrderbook: writable(null),
			subgraphUrl: writable(null),
			orderHash: writable(''),
			activeSubgraphs: writable({}),
			settings: writable({ networks: { network1: {} } }),
			accounts: writable({}),
			activeAccountsItems: writable({}),
			activeOrderStatus: writable(undefined),
			hideZeroBalanceVaults: writable(false),
			activeNetworkRef: writable(''),
			activeOrderbookRef: writable(''),
			activeAccounts: writable({}),
			activeNetworkOrderbooks: writable({}),
			showMyItemsOnly: mockShowMyItemsOnly
		}
	},
	url: {
		pathname: '/vaults'
	}
};

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});


describe('Vaults Page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		(useAccount as Mock).mockReturnValue({
			account: mockAccountStore
		});
		mockPageStore.mockSetSubscribeValue({ ...initialPageState, data: { ...initialPageState.data, stores: { ...initialPageState.data.stores, showMyItemsOnly: mockShowMyItemsOnly } } });
		mockShowMyItemsOnly.set(false);
	});

	it('updates showMyItemsOnly store when account changes', async () => {
		render(VaultsPage);

		expect(get(mockShowMyItemsOnly)).toBe(false);

		const testAccount = '0xabcdef1234567890';
		mockAccountStore.set(testAccount);

		expect(get(mockShowMyItemsOnly)).toBe(testAccount);

		mockAccountStore.set(null);

		expect(get(mockShowMyItemsOnly)).toBe(null);
	});

});