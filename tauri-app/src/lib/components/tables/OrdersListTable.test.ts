import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import OrdersListTable from './OrdersListTable.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import { goto } from '$app/navigation';
import { handleOrderRemoveModal } from '$lib/services/modal';
import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
import type { Order } from '$lib/typeshare/subgraphTypes';

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
    handleOrderRemoveModal: vi.fn(),
  };
});

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

const mockOrders: Order[] = [
  {
    id: 'order1',
    orderHash: 'order1',
    orderBytes: '0x00',
    addEvents: [],
    active: false,
    owner: '0xOwner1',
    timestampAdded: '1625247300',
    inputs: [
      {
        id: '0x00',
        owner: '0x00',
        vaultId: '0x00',
        balance: '100',
        orderbook: { id: '0x00' },
        ordersAsInput: [],
        ordersAsOutput: [],
        balanceChanges: [],
        token: { id: '0x00', address: '0x00', symbol: 'ETH' },
      },
    ],
    outputs: [
      {
        id: '0x00',
        owner: '0x00',
        vaultId: '0x00',
        balance: '100',
        orderbook: { id: '0x00' },
        ordersAsInput: [],
        ordersAsOutput: [],
        balanceChanges: [],
        token: { id: '0x00', address: '0x00', symbol: 'USDC' },
      },
    ],
    orderbook: { id: '0x00' },
  },
  {
    id: 'order2',
    orderHash: 'order2',
    orderBytes: '0x00',
    addEvents: [],
    active: true,
    owner: '0xOwner2',
    timestampAdded: '1625247600',
    inputs: [
      {
        id: '0x00',
        owner: '0x00',
        vaultId: '0x00',
        balance: '100',
        orderbook: { id: '0x00' },
        ordersAsInput: [],
        ordersAsOutput: [],
        balanceChanges: [],
        token: { id: '0x00', address: '0x00', symbol: 'USDT' },
      },
    ],
    outputs: [
      {
        id: '0x00',
        owner: '0x00',
        vaultId: '0x00',
        balance: '100',
        orderbook: { id: '0x00' },
        ordersAsInput: [],
        ordersAsOutput: [],
        balanceChanges: [],
        token: { id: '0x00', address: '0x00', symbol: 'DAI' },
      },
    ],
    orderbook: { id: '0x00' },
  },
];

test('renders the orders list table with correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'orders_list') {
      return mockOrders;
    }
  });

  render(OrdersListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(async () => {
    expect(screen.getByTestId('orderListHeadingActive')).toHaveTextContent('Active');
    expect(screen.getByTestId('orderListHeadingID')).toHaveTextContent('Order');
    expect(screen.getByTestId('orderListHeadingOwner')).toHaveTextContent('Owner');
    expect(screen.getByTestId('orderListHeadingOrderbook')).toHaveTextContent('Orderbook');
    expect(screen.getByTestId('orderListHeadingLastAdded')).toHaveTextContent('Last Added');
    expect(screen.getByTestId('orderListHeadingInputs')).toHaveTextContent('Input Token(s)');
    expect(screen.getByTestId('orderListHeadingOutputs')).toHaveTextContent('Output Token(s)');

    expect(await screen.findAllByTestId('bodyRow')).toHaveLength(2);

    expect(await screen.findAllByTestId('orderListRowActive')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowID')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowOrderbook')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowOwner')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowLastAdded')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowInputs')).toHaveLength(2);
    expect(await screen.findAllByTestId('orderListRowOutputs')).toHaveLength(2);

    expect((await screen.findAllByTestId('orderListRowActive'))[0]).toHaveTextContent('Inactive');
    expect((await screen.findAllByTestId('orderListRowActive'))[1]).toHaveTextContent('Active');
    expect((await screen.findAllByTestId('orderListRowID'))[0]).toHaveTextContent('order...rder1');
    expect((await screen.findAllByTestId('orderListRowID'))[1]).toHaveTextContent('order...rder2');
    expect((await screen.findAllByTestId('orderListRowOwner'))[0]).toHaveTextContent(
      '0xOwn...wner1',
    );
    expect((await screen.findAllByTestId('orderListRowOwner'))[1]).toHaveTextContent(
      '0xOwn...wner2',
    );
    expect((await screen.findAllByTestId('orderListRowLastAdded'))[0]).toHaveTextContent(
      formatTimestampSecondsAsLocal(BigInt(mockOrders[0].timestampAdded)),
    );
    expect((await screen.findAllByTestId('orderListRowLastAdded'))[1]).toHaveTextContent(
      formatTimestampSecondsAsLocal(BigInt(mockOrders[1].timestampAdded)),
    );
    expect((await screen.findAllByTestId('orderListRowInputs'))[0]).toHaveTextContent('ETH');
    expect((await screen.findAllByTestId('orderListRowInputs'))[1]).toHaveTextContent('USDT');
    expect((await screen.findAllByTestId('orderListRowOutputs'))[0]).toHaveTextContent('USDC');
    expect((await screen.findAllByTestId('orderListRowOutputs'))[1]).toHaveTextContent('DAI');
  });
});

test('shows the correct empty message', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'orders_list') {
      return [];
    }
  });

  render(OrdersListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.getByText('No Orders Found')).toBeInTheDocument();
  });
});

test('clicking a row links to the order detail page', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'orders_list') {
      return [mockOrders[0]];
    }
  });

  render(OrdersListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(async () => {
    expect(screen.getByTestId('bodyRow')).toBeInTheDocument();
  });

  await fireEvent.click(await screen.findByTestId('bodyRow'));

  expect(goto).toHaveBeenCalledWith('/orders/order1');
});

test('does not show the dropdown menu if the wallet address does not match', async () => {
  const queryClient = new QueryClient();

  const modifiedMockOrders = [...mockOrders];
  modifiedMockOrders[0].active = true;

  mockIPC((cmd) => {
    if (cmd === 'orders_list') {
      return modifiedMockOrders;
    }
  });

  render(OrdersListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    expect(screen.queryByTestId('order-menu-order1')).not.toBeInTheDocument();
  });

  mockWalletAddressMatchesOrBlankStore.set(() => true);

  await waitFor(() => {
    expect(screen.queryByTestId('order-menu-order1')).toBeInTheDocument();
  });
});

test('clicking the remove option in the dropdown menu opens the remove modal', async () => {
  const queryClient = new QueryClient();

  mockWalletAddressMatchesOrBlankStore.set(() => true);

  const modifiedMockOrders = [...mockOrders];
  modifiedMockOrders[0].active = true;

  mockIPC((cmd) => {
    if (cmd === 'orders_list') {
      return modifiedMockOrders;
    }
  });

  render(OrdersListTable, { context: new Map([['$$_queryClient', queryClient]]) });

  await waitFor(() => {
    screen.getByTestId('order-menu-order1').click();
  });

  await waitFor(() => {
    screen.getByText('Remove').click();
  });

  await waitFor(() => {
    expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrders[0]);
  });
});
