import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import VaultsPage from './+page.svelte';
import {  useAccount } from '@rainlanguage/ui-components';


const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const mockAccountStore = writable('0xabcdef1234567890abcdef1234567890abcdef12');
const mockShowMyItemsOnly = writable(false);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	const MockComponent = (await import('$lib/__mocks__/MockComponent.svelte')).default;
	const original = (await importOriginal()) as object;
	return {
		...original,
		VaultsListTable: MockComponent,
		useAccount: vi.fn()
	};
});



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
	});

	it('updates showMyItemsOnly store when account changes', async () => {
		render(VaultsPage);

		mockPageStore.mockSetSubscribeValue({
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
			}
		});

		const testAccount = '0xabcdef1234567890';
		mockAccountStore.set(testAccount);
		expect(get(mockShowMyItemsOnly)).toBe(true);
		mockAccountStore.set(null);
		expect(get(mockShowMyItemsOnly)).toBe(false);
	});

});