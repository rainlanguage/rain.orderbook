import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultListTable from './VaultListTable.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import { goto } from '$app/navigation';
import {
  handleDepositGenericModal,
  handleDepositModal,
  handleWithdrawModal,
} from '$lib/services/modal';
import type { Vault } from '$lib/typeshare/subgraphTypes';

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
  const _hideZeroBalanceVaults = writable(true);

  return {
    ...((await importOriginal()) as object),
    settings: mockSettingsStore,
    subgraphUrl: writable('https://example.com'),
    activeOrderbook: {
      ..._activeOrderbook,
      load: vi.fn(() => _activeOrderbook.set(true)),
    },
    hideZeroBalanceVaults: _hideZeroBalanceVaults,
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

vi.mock('$app/stores', async () => {
  const { writable } = await import('svelte/store');
  return {
    page: writable({ url: { pathname: '/vaults' } }),
  };
});

test('renders the vault list table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByText('Vault ID')).toBeInTheDocument();
    expect(screen.getByText('Owner')).toBeInTheDocument();
    expect(screen.getByText('Token')).toBeInTheDocument();
    expect(screen.getByText('Balance')).toBeInTheDocument();
    expect(screen.getByText('Input For')).toBeInTheDocument();
    expect(screen.getByText('Output For')).toBeInTheDocument();

    expect(screen.getByTestId('vault-id')).toHaveTextContent('0xabc');
    expect(screen.getByTestId('vault-orderbook')).toHaveTextContent('0x00');
    expect(screen.getByTestId('vault-owner')).toHaveTextContent('0x123');
    expect(screen.getByTestId('vault-token')).toHaveTextContent('USDC coin');
    expect(screen.getByTestId('vault-balance')).toHaveTextContent('100000 USDC');
    expect(screen.queryByTestId('vault-input-for')).not.toBeInTheDocument();
    expect(screen.queryByTestId('vault-output-for')).not.toBeInTheDocument();
  });
});

test('shows the correct empty message', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByText('No Vaults Found')).toBeInTheDocument();
  });
});

test('clicking a row links to the vault detail page', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '0xabc',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
          balanceChanges: [],
        },
      ] as Vault[];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByTestId('vault-id')).toHaveTextContent('0xabc');
  });

  await waitFor(() => {
    screen.getByTestId('bodyRow').click();
  });

  expect(goto).toHaveBeenCalledWith('/vaults/0xabc');
});

test('new vault button is disabled if there is no active orderbook selected', async () => {
  const queryClient = new QueryClient();

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByTestId('new-vault-button')).toBeDisabled();
  });
});

test('clicking the new vault button opens the generic deposit modal', async () => {
  const { activeOrderbook } = await import('$lib/stores/settings');
  activeOrderbook.load();

  const queryClient = new QueryClient();

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    screen.getByTestId('new-vault-button').click();
  });

  await waitFor(() => {
    expect(handleDepositGenericModal).toHaveBeenCalled();
  });
});

test('shows an ellipsis if there is more than three orders as input or output', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '0xabc',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [
            { id: '1', order_id: '0x123', amount: '100000000000' },
            { id: '2', order_id: '0x456', amount: '100000000000' },
            { id: '3', order_id: '0x789', amount: '100000000000' },
            { id: '4', order_id: '0xabc', amount: '100000000000' },
          ],
          ordersAsOutput: [
            { id: '1', order_id: '0x123', amount: '100000000000' },
            { id: '2', order_id: '0x456', amount: '100000000000' },
            { id: '3', order_id: '0x789', amount: '100000000000' },
            { id: '4', order_id: '0xabc', amount: '100000000000' },
          ],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getAllByTestId('vault-order-input').length).toBe(3);
    expect(screen.getAllByTestId('vault-order-output').length).toBe(3);
    expect(screen.getByTestId('vault-order-inputs')).toHaveTextContent('...');
    expect(screen.getByTestId('vault-order-outputs')).toHaveTextContent('...');
  });
});

test('clicking on an order links to the order detail page', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '0xabc',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [{ id: '0x123', order_id: '0x123', amount: '100000000000' }],
          ordersAsOutput: [{ id: '0x456', order_id: '0x456', amount: '100000000000' }],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    screen.getByTestId('vault-order-input').click();
  });

  expect(goto).toHaveBeenCalledWith('/orders/0x123');

  await waitFor(() => {
    screen.getByTestId('vault-order-output').click();
  });

  expect(goto).toHaveBeenCalledWith('/orders/0x456');
});

test('doesnt show the row dropdown menu if the wallet address does not match', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '0xabc',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
          balanceChanges: [],
        },
      ] as Vault[];
    }
  });

  await waitFor(() => {
    expect(screen.queryByTestId('vault-menu')).not.toBeInTheDocument();
  });

  mockWalletAddressMatchesOrBlankStore.set(() => true);

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.queryByTestId('vault-menu')).toBeInTheDocument();
  });
});

test('clicking the deposit option in the row dropdown menu opens the deposit modal', async () => {
  const queryClient = new QueryClient();

  const vault: Vault = {
    id: '0xabc',
    vaultId: '0xabc',
    owner: '0x123',
    token: {
      id: '1',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    ordersAsInput: [],
    ordersAsOutput: [],
    orderbook: { id: '0x00' },
    balanceChanges: [],
  };

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [vault];
    }
  });

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    screen.getByTestId('vault-menu').click();
  });

  await waitFor(() => {
    screen.getByTestId('deposit-button').click();
  });

  await waitFor(() => {
    expect(handleDepositModal).toHaveBeenCalledWith(vault, expect.any(Function));
  });

  await waitFor(() => {
    screen.getByTestId('withdraw-button').click();
  });

  await waitFor(() => {
    expect(handleWithdrawModal).toHaveBeenCalledWith(vault, expect.any(Function));
  });
});

test('hides zero balance vaults when hideZeroBalanceVaults is true', async () => {
  const queryClient = new QueryClient();
  const { hideZeroBalanceVaults } = await import('$lib/stores/settings');

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  hideZeroBalanceVaults.set(true);

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByTestId('vault-id')).toHaveTextContent('0xabc');
  });
});

test('shows all vaults when hideZeroBalanceVaults is false', async () => {
  const queryClient = new QueryClient();
  const { hideZeroBalanceVaults } = await import('$lib/stores/settings');

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
        {
          id: '2',
          vaultId: '0xdef',
          owner: '0x456',
          token: {
            id: '2',
            address: '0x789',
            name: 'ETH coin',
            symbol: 'ETH',
            decimals: '18',
          },
          balance: '0',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  hideZeroBalanceVaults.set(false);

  render(VaultListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByText('0xabc')).toBeInTheDocument();
    expect(screen.getByText('0xdef')).toBeInTheDocument();
  });
});

test('updates the vault list when hideZeroBalanceVaults changes', async () => {
  const queryClient = new QueryClient();
  const { hideZeroBalanceVaults } = await import('$lib/stores/settings');

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  hideZeroBalanceVaults.set(true);

  render(VaultListTable, {
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getAllByTestId('vault-id')).toHaveLength(1);
    expect(screen.getByTestId('vault-id')).toHaveTextContent('0xabc');
  });

  hideZeroBalanceVaults.set(false);

  mockIPC((cmd) => {
    if (cmd === 'vaults_list') {
      return [
        {
          id: '1',
          vaultId: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
        {
          id: '2',
          vaultId: '0xdef',
          owner: '0x456',
          token: {
            id: '2',
            address: '0x789',
            name: 'ETH coin',
            symbol: 'ETH',
            decimals: '18',
          },
          balance: '0',
          ordersAsInput: [],
          ordersAsOutput: [],
          orderbook: { id: '0x00' },
        },
      ];
    }
  });

  await waitFor(() => {
    expect(screen.getAllByTestId('vault-id')).toHaveLength(2);
    expect(screen.getAllByTestId('vault-id')[0]).toHaveTextContent('0xabc');
    expect(screen.getAllByTestId('vault-id')[1]).toHaveTextContent('0xdef');
  });
});
