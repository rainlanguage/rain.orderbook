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
            name: 'USDC coin',
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
    expect(screen.getByText('Owner')).toBeInTheDocument();
    expect(screen.getByText('Token')).toBeInTheDocument();
    expect(screen.getByText('Balance')).toBeInTheDocument();
    expect(screen.getByText('Input For')).toBeInTheDocument();
    expect(screen.getByText('Output For')).toBeInTheDocument();

    expect(screen.getByTestId('vault-id')).toHaveTextContent('0xabc');
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
          vault_id: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
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
          vault_id: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          orders_as_input: [
            { id: '1', order_id: '0x123', amount: '100000000000' },
            { id: '2', order_id: '0x456', amount: '100000000000' },
            { id: '3', order_id: '0x789', amount: '100000000000' },
            { id: '4', order_id: '0xabc', amount: '100000000000' },
          ],
          orders_as_output: [
            { id: '1', order_id: '0x123', amount: '100000000000' },
            { id: '2', order_id: '0x456', amount: '100000000000' },
            { id: '3', order_id: '0x789', amount: '100000000000' },
            { id: '4', order_id: '0xabc', amount: '100000000000' },
          ],
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
          vault_id: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
            symbol: 'USDC',
            decimals: '6',
          },
          balance: '100000000000',
          orders_as_input: [{ id: '0x123', order_id: '0x123', amount: '100000000000' }],
          orders_as_output: [{ id: '0x456', order_id: '0x456', amount: '100000000000' }],
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
          vault_id: '0xabc',
          owner: '0x123',
          token: {
            id: '1',
            address: '0x456',
            name: 'USDC coin',
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

  const vault = {
    id: '0xabc',
    vault_id: '0xabc',
    owner: '0x123',
    token: {
      id: '1',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    orders_as_input: [],
    orders_as_output: [],
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
    expect(handleDepositModal).toHaveBeenCalledWith(vault);
  });

  await waitFor(() => {
    screen.getByTestId('withdraw-button').click();
  });

  await waitFor(() => {
    expect(handleWithdrawModal).toHaveBeenCalledWith(vault);
  });
});
