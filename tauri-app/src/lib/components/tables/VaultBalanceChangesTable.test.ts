import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import { mockIPC } from '@tauri-apps/api/mocks';
import VaultBalanceChangesTable from './VaultBalanceChangesTable.svelte';
import type { VaultBalanceChange } from '$lib/typeshare/vaultBalanceChangesList';
import { formatTimestampSecondsAsLocal } from '$lib/utils/time';

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

test('renders the vault list table with correct data', async () => {
  const queryClient = new QueryClient();

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
          decimals: '4',
        },
      },
      timestamp: '1625247600',
      transaction: {
        id: 'tx1',
        from: '0xUser1',
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
