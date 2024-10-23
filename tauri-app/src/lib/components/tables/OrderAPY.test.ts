import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { OrderAPY } from '$lib/typeshare/subgraphTypes';
import { QueryClient } from '@tanstack/svelte-query';
import OrderApy from './OrderAPY.svelte';
import { bigintStringToPercentage } from '$lib/utils/number';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { writable } = await import('svelte/store');
  const { mockSettingsStore } = await import('$lib/mocks/settings');

  const _activeOrderbook = writable();

  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    subgraphUrl: writable('https://example.com'),
    activeOrderbook: {
      ..._activeOrderbook,
      load: vi.fn(() => _activeOrderbook.set(true)),
    },
  };
});

vi.mock('$lib/services/modal', async () => {
  return {
    handleDepositGenericModal: vi.fn(),
    handleDepositModal: vi.fn(),
    handleWithdrawModal: vi.fn(),
  };
});

const mockOrderApy: OrderAPY[] = [
  {
    orderId: '1',
    orderHash: '1',
    denominatedApy: {
      apy: '1200000000000000000',
      token: {
        id: 'output_token',
        address: 'output_token',
        name: 'output_token',
        symbol: 'output_token',
        decimals: '0',
      },
    },
    startTime: 1,
    endTime: 2,
    inputsTokenVaultApy: [],
    outputsTokenVaultApy: [],
  },
];

test('renders table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'order_apy') {
      return mockOrderApy[0];
    }
  });

  render(OrderApy, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { id: '1' },
  });

  await waitFor(async () => {
    // get apy row
    const rows = screen.getAllByTestId('apy-field');

    // checking
    for (let i = 0; i < mockOrderApy.length; i++) {
      const display = bigintStringToPercentage(mockOrderApy[i].denominatedApy!.apy, 18, 5);
      expect(rows[i]).toHaveTextContent(display);
    }
  });
});
