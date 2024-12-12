import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import VaultDetail from './VaultDetail.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Vault } from '$lib/typeshare/subgraphTypes';
import { goto } from '$app/navigation';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';

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
    props: { id: '100', network: 'mainnet' },
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
    props: { id: '100', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => expect(screen.getByTestId('emptyMessage')).toBeInTheDocument());
});

test('shows the correct data when the query returns data', async () => {
  const mockData: Vault = {
    id: '1',
    vaultId: '0xabc',
    owner: '0x123',
    token: {
      id: '0x456',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    ordersAsInput: [],
    ordersAsOutput: [],
    balanceChanges: [],
    orderbook: {
      id: '0x00',
    },
  };
  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getByTestId('vaultDetailTokenName')).toHaveTextContent('USDC coin');
    expect(screen.getByTestId('vaultDetailVaultId')).toHaveTextContent('Vault ID 0xabc');
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

test('shows the correct data when the query returns data with orders', async () => {
  const mockData: Vault = {
    id: '1',
    vaultId: '0xabc',
    owner: '0x123',
    token: {
      id: '0x456',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    ordersAsInput: [
      {
        id: '1',
        orderHash: '0x123',
        active: true,
      },
      {
        id: '2',
        orderHash: '0x456',
        active: false,
      },
    ],
    ordersAsOutput: [
      {
        id: '3',
        orderHash: '0x789',
        active: true,
      },
      {
        id: '4',
        orderHash: '0xabc',
        active: false,
      },
    ],
    balanceChanges: [],
    orderbook: {
      id: '0x00',
    },
  };
  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(async () => {
    expect(
      await screen.findAllByTestId('vaultDetailOrderAsInputOrder', { exact: false }),
    ).toHaveLength(2);
    expect(
      await screen.findAllByTestId('vaultDetailOrderAsOutputOrder', { exact: false }),
    ).toHaveLength(2);
  });

  const orderAsInputOrders = screen.getAllByTestId('vaultDetailOrderAsInputOrder', {
    exact: false,
  });
  expect(orderAsInputOrders[0]).toHaveTextContent('0x123...0x123');
  expect(orderAsInputOrders[1]).toHaveTextContent('0x456...0x456');

  const orderAsOutputOrders = screen.getAllByTestId('vaultDetailOrderAsOutputOrder', {
    exact: false,
  });

  expect(orderAsOutputOrders[0]).toHaveTextContent('0x789...0x789');
  expect(orderAsOutputOrders[1]).toHaveTextContent('0xabc...0xabc');
});

test('orders link to the correct order', async () => {
  const mockData: Vault = {
    id: '1',
    vaultId: '0xabc',
    owner: '0x123',
    token: {
      id: '0x456',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    ordersAsInput: [
      {
        id: '1',
        orderHash: '0x123',
        active: true,
      },
      {
        id: '2',
        orderHash: '0x456',
        active: false,
      },
    ],
    ordersAsOutput: [
      {
        id: '3',
        orderHash: '0x789',
        active: true,
      },
      {
        id: '4',
        orderHash: '0xabc',
        active: false,
      },
    ],
    balanceChanges: [],
    orderbook: {
      id: '0x00',
    },
  };
  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  let ordersAsOutput, ordersAsInput;

  await waitFor(async () => {
    ordersAsInput = await screen.findAllByTestId('vaultDetailOrderAsInputOrder', { exact: false });
    await Promise.all(
      ordersAsInput.map(async (order) => {
        await order.click();
        expect(goto).toHaveBeenCalledWith(`/orders/${order.getAttribute('data-order')}`);
      }),
    );
  });

  await waitFor(async () => {
    ordersAsOutput = await screen.findAllByTestId('vaultDetailOrderAsOutputOrder', {
      exact: false,
    });
    await Promise.all(
      ordersAsOutput.map(async (order) => {
        await order.click();
        expect(goto).toHaveBeenCalledWith(`/orders/${order.getAttribute('data-order')}`);
      }),
    );
  });
});

test('shows deposit and withdraw buttons if owner wallet matches, opens correct modals', async () => {
  const mockData: Vault = {
    id: '1',
    vaultId: '0xabc',
    owner: '0x123',
    token: {
      id: '0x456',
      address: '0x456',
      name: 'USDC coin',
      symbol: 'USDC',
      decimals: '6',
    },
    balance: '100000000000',
    ordersAsInput: [],
    ordersAsOutput: [],
    balanceChanges: [],
    orderbook: {
      id: '0x00',
    },
  };

  mockIPC((cmd) => {
    if (cmd === 'vault_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(VaultDetail, {
    props: { id: '100', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.queryByTestId('vaultDetailDepositButton')).not.toBeInTheDocument();
    expect(screen.queryByTestId('vaultDetailWithdrawButton')).not.toBeInTheDocument();
  });

  mockWalletAddressMatchesOrBlankStore.set(() => true);

  await waitFor(() => {
    expect(screen.queryByTestId('vaultDetailDepositButton')).toBeInTheDocument();
    expect(screen.queryByTestId('vaultDetailWithdrawButton')).toBeInTheDocument();
  });

  screen.getByTestId('vaultDetailDepositButton').click();

  await waitFor(() => {
    expect(handleDepositModal).toHaveBeenCalledWith(mockData, expect.any(Function));
  });

  screen.getByTestId('vaultDetailWithdrawButton').click();

  await waitFor(() => {
    expect(handleWithdrawModal).toHaveBeenCalledWith(mockData, expect.any(Function));
  });
});
