import { render, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { writable, type Writable } from 'svelte/store';
import Page from './+page.svelte';
import {
  resetActiveNetworkRef,
  resetActiveOrderbookRef,
  activeOrderbook,
} from '$lib/stores/settings';
import type { OrderbookConfigSource } from '@rainlanguage/ui-components';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  const internalMockActiveOrderbook = writable<OrderbookConfigSource | null | undefined>(null);
  return {
    ...original,
    activeOrderbook: internalMockActiveOrderbook, // Return the internally defined store
    resetActiveNetworkRef: vi.fn(),
    resetActiveOrderbookRef: vi.fn(),
  };
});

vi.mock('$app/stores', async () => {
  return {
    page: mockPageStore,
  };
});

vi.mock('@rainlanguage/ui-components', async () => {
  const MockComponent = (await import('../../lib/__mocks__/MockComponent.svelte')).default;
  return {
    OrdersListTable: MockComponent,
    PageHeader: MockComponent,
  };
});

describe('routes/orders/+page.svelte', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockPageStore.reset();

    (
      vi.mocked(activeOrderbook) as unknown as Writable<OrderbookConfigSource | null | undefined>
    ).set(null);
  });

  it('should NOT call reset functions if activeOrderbook store is truthy on mount', async () => {
    (
      vi.mocked(activeOrderbook) as unknown as Writable<OrderbookConfigSource | null | undefined>
    ).set({} as OrderbookConfigSource);
    render(Page);

    await waitFor(() => {
      expect(resetActiveNetworkRef as Mock).not.toHaveBeenCalled();
      expect(resetActiveOrderbookRef as Mock).not.toHaveBeenCalled();
    });
  });

  it('should call reset functions if activeOrderbook store is falsy on mount', async () => {
    render(Page);

    await waitFor(() => {
      expect(resetActiveNetworkRef as Mock).toHaveBeenCalledTimes(1);
      expect(resetActiveOrderbookRef as Mock).toHaveBeenCalledTimes(1);
    });
  });
});
