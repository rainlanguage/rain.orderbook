import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultListTable from './VaultListTable.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';

vi.mock('$lib/stores/wallets', async () => {
  const { writable } = await import('svelte/store');
  return {
    walletAddressMatchesOrBlank: writable(() => false),
  };
});

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const { writable } = await import('svelte/store');
  const { mockSettingsStore } = await import('$lib/stores/settings.test');
  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    subgraphUrl: writable('https://example.com'),
  };
});

test('renders the vault list table with correct data', async () => {
  const queryClient = new QueryClient();
  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vault_id: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          orders_as_input: [],
          orders_as_output: [],
        },
      ];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByText('Vault ID')).toBeInTheDocument();
  });
});
