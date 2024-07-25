import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from './VaultDetail.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Vault } from '$lib/typeshare/vaultDetail';
// import { goto } from '$app/navigation';
// import {
//   handleDepositGenericModal,
//   handleDepositModal,
//   handleWithdrawModal,
// } from '$lib/services/modal';

const { mockWalletAddressMatchesOrBlankStore } = await vi.hoisted(
  () => import('$lib/mocks/wallets'),
);

vi.mock('$lib/stores/wallets', async () => {
  return {
    walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
  };
});

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

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

vi.mock('lightweight-charts', async (importOriginal) => ({
  ...((await importOriginal()) as object),
  createChart: vi.fn(() => ({
    addLineSeries: vi.fn(),
    remove(): void {},
    applyOptions: vi.fn(),
  })),
}));

test('calls the vault detail query fn with the correct vault id', async () => {
  let receivedId: string;
  mockIPC((cmd, args) => {
    if (cmd === 'vault_detail') {
      receivedId = args.id as string;
      return [];
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => expect(receivedId).toEqual('100'));
});

test('shows the correct empty message when the query returns no data', async () => {
  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return null;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => expect(screen.getByTestId('emptyMessage')).toBeInTheDocument());
});

test('shows the correct data when the query returns data', async () => {
  const mockData: Vault = {
    id: '1',
    vault_id: '1',
    owner: '0x123',
    token: {
      id: '0x456',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    orders_as_input: [],
    orders_as_output: [],
    balance_changes: [],
  };
  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getByTestId('vaultDetailTokenName')).toHaveTextContent('USDC coin');
    expect(screen.getByTestId('vaultDetailOwnerAddress')).toHaveTextContent(
      'Owner Address 0x123...0x123',
    );
    expect(screen.getByTestId('vaultDetailTokenAddress')).toHaveTextContent(
      'Token address 0x456...0x456',
    );
    expect(screen.getByTestId('vaultDetailBalance')).toHaveTextContent('Balance 100000 USDC');
    expect(screen.queryByTestId('vaultDetailOrdersAsInput')).toHaveTextContent('None');
    expect(screen.queryByTestId('vaulDetailOrdersAsOutput')).toHaveTextContent('None');
  });
});
