import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { VaultVolume } from '$lib/typeshare/subgraphTypes';
import { formatUnits } from 'viem';
import OrderVaultsVolTable from './OrderVaultsVolTable.svelte';
import { QueryClient } from '@tanstack/svelte-query';

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { writable } = await import('svelte/store');
  const { mockSettingsStore } = await import('@rainlanguage/ui-components');

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

const mockVaultsVol: VaultVolume[] = [
  {
    id: '1',
    token: {
      id: 'output_token',
      address: 'output_token',
      name: 'output_token',
      symbol: 'output_token',
      decimals: '0',
    },
    totalIn: '1',
    totalOut: '2',
    totalVol: '3',
    netVol: '-1',
  },
  {
    id: '2',
    token: {
      id: 'output_token',
      address: 'output_token',
      name: 'output_token',
      symbol: 'output_token',
      decimals: '0',
    },
    totalIn: '2',
    totalOut: '5',
    totalVol: '7',
    netVol: '-3',
  },
];

test('renders table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'order_vaults_volume') {
      return mockVaultsVol;
    }
  });

  render(OrderVaultsVolTable, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { id: '1' },
  });

  await waitFor(async () => {
    // get total ins
    const rows = screen.getAllByTestId('total-in');

    // checking the total ins
    for (let i = 0; i < mockVaultsVol.length; i++) {
      const display = formatUnits(
        BigInt(mockVaultsVol[i].totalIn),
        Number(mockVaultsVol[i].token.decimals),
      );
      expect(rows[i]).toHaveTextContent(display.toString());
    }
  });

  await waitFor(async () => {
    // get total outs
    const rows = screen.getAllByTestId('total-out');

    // checking the total outs
    for (let i = 0; i < mockVaultsVol.length; i++) {
      const display = formatUnits(
        BigInt(mockVaultsVol[i].totalOut),
        Number(mockVaultsVol[i].token.decimals),
      );
      expect(rows[i]).toHaveTextContent(display.toString());
    }
  });

  await waitFor(async () => {
    // get net vols
    const rows = screen.getAllByTestId('net-vol');

    // checking the net vols
    for (let i = 0; i < mockVaultsVol.length; i++) {
      const display = formatUnits(
        BigInt(mockVaultsVol[i].netVol),
        Number(mockVaultsVol[i].token.decimals),
      );
      expect(rows[i]).toHaveTextContent(display.toString());
    }
  });

  await waitFor(async () => {
    // get total vols
    const rows = screen.getAllByTestId('total-vol');

    // checking the total vols
    for (let i = 0; i < mockVaultsVol.length; i++) {
      const display = formatUnits(
        BigInt(mockVaultsVol[i].totalVol),
        Number(mockVaultsVol[i].token.decimals),
      );
      expect(rows[i]).toHaveTextContent(display.toString());
    }
  });
});
