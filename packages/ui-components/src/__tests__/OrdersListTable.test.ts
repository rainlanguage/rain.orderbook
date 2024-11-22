import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import OrdersListTable from '../lib/components/tables/OrdersListTable.svelte';
import { getOrders } from '@rainlanguage/orderbook/js_api';
import { readable } from 'svelte/store';

// Mock getOrders
vi.mock('@rainlanguage/orderbook/js_api', () => ({
  getOrders: vi.fn()
}));

// Hoisted mock stores
const {
  mockActiveNetworkRefStore,
  mockActiveOrderbookRefStore,
  mockOrderHashStore,
  mockAccountsStore,
  mockActiveAccountsItemsStore,
  mockActiveOrderStatusStore,
  mockActiveSubgraphsStore,
  mockSettingsStore,
  mockActiveAccountsStore
} = await vi.hoisted(() => import('../lib/__mocks__/stores'));

const mockOrder = {
  order: {
    id: "0x1234567890abcdef1234567890abcdef12345678",
    orderHash: "0x2222222222222222222222222222222222222222",
    owner: "0xabcdef1234567890abcdef1234567890abcdef12",
    active: true,
    timestampAdded: "1677777777",
    orderbook: {
      id: "0x3333333333333333333333333333333333333333"
    },
    inputs: [{ token: { symbol: "ETH" } }],
    outputs: [{ token: { symbol: "USDC" } }],
    trades: []
  },
  subgraphName: "mock-subgraph-mainnet"
};

const defaultProps = {
  activeSubgraphs: mockActiveSubgraphsStore,
  settings: mockSettingsStore,
  accounts: mockAccountsStore,
  activeAccountsItems: mockActiveAccountsItemsStore,
  activeOrderStatus: mockActiveOrderStatusStore,
  orderHash: mockOrderHashStore,
  hideZeroBalanceVaults: readable(false),
  currentRoute: '/orders',
  activeNetworkRef: mockActiveNetworkRefStore,
  activeOrderbookRef: mockActiveOrderbookRefStore
};

describe('OrdersListTable', () => {
  beforeEach(() => {
    vi.mocked(getOrders).mockReset();
    vi.mocked(getOrders).mockResolvedValue([mockOrder]);
  });

  it('renders order list with mock data', async () => {
    render(OrdersListTable, defaultProps);

    await screen.findByTestId('orderListRowNetwork');

    expect(screen.getByTestId('orderListRowNetwork')).toHaveTextContent('mock-subgraph-mainnet');
    expect(screen.getByTestId('orderListRowActive')).toHaveTextContent('Active');
    expect(screen.getByTestId('orderListRowInputs')).toHaveTextContent('ETH');
    expect(screen.getByTestId('orderListRowOutputs')).toHaveTextContent('USDC');
  });
});