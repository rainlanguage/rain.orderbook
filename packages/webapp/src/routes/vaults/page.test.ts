import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { render } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import VaultsPage from './+page.svelte';
import { useAccount } from '@rainlanguage/ui-components';

const { mockPageStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));

const mockAccountStore = vi.fn();

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {

	const mockComponent = (await import('@rainlanguage/ui-components')).default
	return {
		...(await importOriginal()),
		VaultsListTable: mockComponent,
		useAccount: vi.fn()
	}
})

// Mock the page store
const mockShowMyItemsOnly = writable(false);
const mockPageData = {
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
};

vi.mock('$app/stores', () => ({
  page: mockPageStore
}));

describe('Vaults Page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
(useAccount as Mock).mockReturnValue({
			account: mockAccountStore
		});
    mockShowMyItemsOnly.set(false);
  });


  it('updates showMyItemsOnly store when account changes', async () => {
    render(VaultsPage);
    
    expect(get(mockShowMyItemsOnly)).toBe(false);
    
    const testAccount = '0xabcdef1234567890';
	mockAccountStore.mockReturnValue(writable(testAccount));
    
    expect(get(mockShowMyItemsOnly)).toBe(testAccount);
    
    mockAccountStore.mockReturnValue(writable(null));
    
    expect(get(mockShowMyItemsOnly)).toBe(null);
  });

  it('initializes network and orderbook refs when no active orderbook', async () => {
    const resetActiveNetworkRefSpy = vi.fn();
    const resetActiveOrderbookRefSpy = vi.fn();
    
    const CustomVaultsPage = {
      ...VaultsPage,
      resetActiveNetworkRef: resetActiveNetworkRefSpy,
      resetActiveOrderbookRef: resetActiveOrderbookRefSpy
    };
    
    mockPageData.stores.activeOrderbook.set(null);
    
    render(CustomVaultsPage);
    
    await vi.waitFor(() => {
      expect(resetActiveNetworkRefSpy).toHaveBeenCalled();
      expect(resetActiveOrderbookRefSpy).toHaveBeenCalled();
    });
  });

  it('does not initialize network and orderbook refs when active orderbook exists', async () => {
    const resetActiveNetworkRefSpy = vi.fn();
    const resetActiveOrderbookRefSpy = vi.fn();
    
    const CustomVaultsPage = {
      ...VaultsPage,
      resetActiveNetworkRef: resetActiveNetworkRefSpy,
      resetActiveOrderbookRef: resetActiveOrderbookRefSpy
    };
    	 mockPageData.stores.activeOrderbook.set(null);
    
    render(CustomVaultsPage);
    
    expect(resetActiveNetworkRefSpy).not.toHaveBeenCalled();
    expect(resetActiveOrderbookRefSpy).not.toHaveBeenCalled();
  });
});