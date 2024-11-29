import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import { mockIPC } from '@tauri-apps/api/mocks';
import VaultBalanceChangesTable from './VaultBalanceChangesTable.svelte';
import type { VaultBalanceChangeUnwrapped } from '$lib/typeshare/subgraphTypes';
import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';

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

test('renders the vault list table with correct data', async () => {
  const queryClient = new QueryClient();

  const mockVaultBalanceChanges: VaultBalanceChangeUnwrapped[] = [
    {
      typename: 'Withdrawal',
      amount: '1000',
      oldVaultBalance: '5000',
      newVaultBalance: '4000',
      vault: {
        id: 'vault1',
        vaultId: 'vault-id1',
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
      typename: 'TradeVaultBalanceChange',
      amount: '1500',
      oldVaultBalance: '4000',
      newVaultBalance: '2500',
      vault: {
        id: 'vault2',
        vaultId: 'vault-id2',
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
        vaultId: 'vault-id3',
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

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChanges;
    }
  });

  render(VaultBalanceChangesTable, {
    props: { id: '100' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    const rows = screen.getAllByTestId('bodyRow');
    expect(rows).toHaveLength(3);
  });
});

test('it shows the correct data in the table', async () => {
  const queryClient = new QueryClient();

  const mockVaultBalanceChanges: VaultBalanceChangeUnwrapped[] = [
    {
      typename: 'Withdrawal',
      amount: '1000',
      oldVaultBalance: '5000',
      newVaultBalance: '4000',
      vault: {
        id: 'vault1',
        vaultId: 'vault-id1',
        token: {
          id: 'token1',
          address: '0xTokenAddress1',
          name: 'Token1',
          symbol: 'TKN1',
          decimals: '4',
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
  ];

  mockIPC((cmd) => {
    if (cmd === 'vault_balance_changes_list') {
      return mockVaultBalanceChanges;
    }
  });

  render(VaultBalanceChangesTable, {
    props: { id: '100' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getByTestId('vaultBalanceChangesTableDate')).toHaveTextContent(
      formatTimestampSecondsAsLocal(BigInt('1625247600')),
    );
    expect(screen.getByTestId('vaultBalanceChangesTableFrom')).toHaveTextContent('0xUse...User1');
    expect(screen.getByTestId('vaultBalanceChangesTableTx')).toHaveTextContent('tx1');
    expect(screen.getByTestId('vaultBalanceChangesTableBalanceChange')).toHaveTextContent('1 TKN1');
    expect(screen.getByTestId('vaultBalanceChangesTableBalance')).toHaveTextContent('0.4 TKN1');
    expect(screen.getByTestId('vaultBalanceChangesTableType')).toHaveTextContent('Withdrawal');
  });
});
