import { describe, it, expect, vi } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import OrdersPage from './+page.svelte';
// import { resetActiveNetworkRef, resetActiveOrderbookRef } from '$lib/stores/settings';

// vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
//   const MockComponent = (await import('@rainlanguage/ui-components')).MockComponent;
//   return {
//     ...(await importOriginal()),
//     OrdersListTable: MockComponent,
// 	PageHeader: MockComponent,
//   };
// });

// vi.mock('$lib/services/modal', () => ({
//   handleOrderRemoveModal: vi.fn(),
// }));

const mockActiveOrderBook = vi.hoisted(() => writable(undefined));

// vi.mock('$lib/stores/settings', () => {
//   return {
//     activeSubgraphs: writable({}),
//     settings: writable({}),
//     accounts: writable({}),
//     activeAccountsItems: writable({}),
//     activeOrderStatus: writable<boolean | undefined>(undefined),
//     orderHash: writable(''),
//     hideZeroBalanceVaults: writable(false),
//     resetActiveNetworkRef: vi.fn(),
//     resetActiveOrderbookRef: vi.fn(),
//     activeOrderbook: mockActiveOrderBook,
//     activeNetworkRef: writable(''),
//     activeOrderbookRef: writable(''),
//     activeNetworkOrderbooks: writable(''),
//     orderbookAddress: writable(''),
//   };
// });

describe('Orders Page (tauri-app/src/routes/orders/+page.svelte)', () => {
  it('calls reset functions on mount if activeOrderbook is initially falsy', async () => {
    render(OrdersPage);

    await waitFor(() => {
      expect(resetActiveNetworkRef).toHaveBeenCalledTimes(1);
      expect(resetActiveOrderbookRef).toHaveBeenCalledTimes(1);
    });
  });

  it('does NOT call reset functions on mount if activeOrderbook is initially truthy', async () => {
    render(OrdersPage);
    expect(resetActiveNetworkRef).not.toHaveBeenCalled();
    expect(resetActiveOrderbookRef).not.toHaveBeenCalled();
  });
});
