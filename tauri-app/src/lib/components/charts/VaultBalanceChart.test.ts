import { render, screen, waitFor } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import { mockIPC } from '@tauri-apps/api/mocks';
import VaultBalanceChart from './VaultBalanceChart.svelte';
import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
import { bigintToFloat } from '$lib/utils/number';
import type { Vault } from '$lib/typeshare/vaultDetail';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';

// Mock settings and subgraphUrl
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

const mockVaultBalanceChanges: VaultBalanceChange[] = [
  {
    __typename: 'Withdrawal',
    amount: '1000',
    old_vault_balance: '5000',
    new_vault_balance: '4000',
    vault: {
      id: 'vault1',
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
    },
  },
  {
    __typename: 'TradeVaultBalanceChange',
    amount: '1500',
    old_vault_balance: '4000',
    new_vault_balance: '2500',
    vault: {
      id: 'vault2',
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
    },
  },
  {
    __typename: 'Deposit',
    amount: '2000',
    old_vault_balance: '2500',
    new_vault_balance: '4500',
    vault: {
      id: 'vault3',
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
    },
  },
];

const mockVault: Vault = {
  id: 'vault1',
  vault_id: 'vault1',
  token: {
    id: 'token1',
    address: '0xTokenAddress1',
    name: 'Token1',
    symbol: 'TKN1',
    decimals: '18',
  },
  owner: '0xOwnerAddress',
  orders_as_output: [],
  orders_as_input: [],
  balance_changes: [],
  balance: '1000000000000000000',
};

test('renders the chart with correct data and transformations', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChanges;
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
        value: bigintToFloat(BigInt(mockVaultBalanceChanges[0].new_vault_balance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChanges[0].timestamp)),
      },
      {
        value: bigintToFloat(BigInt(mockVaultBalanceChanges[1].new_vault_balance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChanges[1].timestamp)),
      },
      {
        value: bigintToFloat(BigInt(mockVaultBalanceChanges[2].new_vault_balance), 18),
        time: timestampSecondsToUTCTimestamp(BigInt(mockVaultBalanceChanges[2].timestamp)),
      },
    ]);
  });
});

test('renders the empty message correctly', async () => {
  const queryClient = new QueryClient();

  const mockVaultBalanceChanges: VaultBalanceChange[] = [];

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChanges;
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
