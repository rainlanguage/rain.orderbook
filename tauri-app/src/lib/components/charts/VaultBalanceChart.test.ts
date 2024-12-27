import { render, screen, waitFor } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import { mockIPC } from '@tauri-apps/api/mocks';
import VaultBalanceChart from './VaultBalanceChart.svelte';
import { timestampSecondsToUTCTimestamp } from '@rainlanguage/ui-components';
import { bigintToFloat } from '$lib/utils/number';
import type { Vault } from '$lib/typeshare/subgraphTypes';
import type { VaultBalanceChangeUnwrapped } from '$lib/typeshare/subgraphTypes';

// Mock settings and subgraphUrl
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

// Mock lightweight-charts library
const setDataMock = vi.fn();

vi.mock('lightweight-charts', async () => {
  const actual = (await vi.importActual('lightweight-charts')) as object;
  return {
    ...actual,
    createChart: vi.fn(() => ({
      addLineSeries: vi.fn(() => ({
        setData: setDataMock,
      })),
      remove: vi.fn(),
      applyOptions: vi.fn(),
      timeScale: vi.fn(() => ({
        setVisibleRange: vi.fn(),
      })),
    })),
  };
});

const mockVaultBalanceChangeUnwrappeds: VaultBalanceChangeUnwrapped[] = [
  {
    typename: 'Withdrawal',
    amount: '1000',
    oldVaultBalance: '5000',
    newVaultBalance: '4000',
    vault: {
      id: 'vault1',
      vault_id: 'vault-id1',
      token: {
        id: 'token1',
        address: '0xTokenAddress1',
        name: 'Token1',
        symbol: 'TKN1',
        decimals: '18',
      },
    },
    timestamp: '1625247600',
    transaction: {
      id: 'tx1',
      from: '0xUser1',
      timestamp: '0',
      blockNumber: '0',
    },
    orderbook: {
      id: '0x00',
    },
  },
  {
    typename: 'TradeVaultBalanceChangeUnwrapped',
    amount: '1500',
    oldVaultBalance: '4000',
    newVaultBalance: '2500',
    vault: {
      id: 'vault2',
      vault_id: 'vault-id2',
      token: {
        id: 'token2',
        address: '0xTokenAddress2',
        name: 'Token2',
        symbol: 'TKN2',
        decimals: '18',
      },
    },
    timestamp: '1625347600',
    transaction: {
      id: 'tx2',
      from: '0xUser2',
      timestamp: '0',
      blockNumber: '0',
    },
    orderbook: {
      id: '0x00',
    },
  },
  {
    typename: 'Deposit',
    amount: '2000',
    oldVaultBalance: '2500',
    newVaultBalance: '4500',
    vault: {
      id: 'vault3',
      vault_id: 'vault-id3',
      token: {
        id: 'token3',
        address: '0xTokenAddress3',
        name: 'Token3',
        symbol: 'TKN3',
        decimals: '18',
      },
    },
    timestamp: '1625447600',
    transaction: {
      id: 'tx3',
      from: '0xUser3',
      timestamp: '0',
      blockNumber: '0',
    },
    orderbook: {
      id: '0x00',
    },
  },
];

const mockVault: Vault = {
  id: 'vault1',
  vaultId: 'vault1',
  token: {
    id: 'token1',
    address: '0xTokenAddress1',
    name: 'Token1',
    symbol: 'TKN1',
    decimals: '18',
  },
  owner: '0xOwnerAddress',
  ordersAsInput: [],
  ordersAsOutput: [],
  balanceChanges: [],
  balance: '1000000000000000000',
  orderbook: {
    id: '0x00',
  },
};

test('renders the chart with correct data and transformations', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChangeUnwrappeds;
    }
  });

  render(VaultBalanceChart, {
    props: { vault: mockVault },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  // Wait for the data to be loaded and rendered
  await waitFor(() => {
    expect(screen.getByText('Balance history')).toBeInTheDocument();
    expect(screen.queryByTestId('loading-spinner')).not.toBeInTheDocument();
  });

  // Check that the setData function is called with the correctly transformed data
  await waitFor(() => {
    expect(setDataMock).toHaveBeenCalledWith([
      {
        value: bigintToFloat(BigInt(mockVaultBalanceChangeUnwrappeds[0].newVaultBalance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChangeUnwrappeds[0].timestamp)),
      },
      {
        value: bigintToFloat(BigInt(mockVaultBalanceChangeUnwrappeds[1].newVaultBalance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChangeUnwrappeds[1].timestamp)),
      },
      {
        value: bigintToFloat(BigInt(mockVaultBalanceChangeUnwrappeds[2].newVaultBalance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChangeUnwrappeds[2].timestamp)),
      },
    ]);
  });
});

test('renders the empty message correctly', async () => {
  const queryClient = new QueryClient();

  const mockVaultBalanceChangeUnwrappeds: VaultBalanceChangeUnwrapped[] = [];

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChangeUnwrappeds;
    }
  });

  render(VaultBalanceChart, {
    props: { vault: mockVault },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  // Wait for the empty message to be rendered
  await waitFor(() => {
    expect(screen.getByText('No deposits or withdrawals found')).toBeInTheDocument();
  });
});
