import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import OrderDetail from './OrderDetail.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import { mockOrderDetailsExtended } from '$lib/queries/orderDetail';
import { handleOrderRemoveModal } from '$lib/services/modal';
import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';
import { formatUnits } from 'viem';

const { mockWalletAddressMatchesOrBlankStore } = await vi.hoisted(
  () => import('$lib/mocks/wallets'),
);

vi.mock('$lib/stores/wallets', async () => {
  return {
    walletAddressMatchesOrBlank: mockWalletAddressMatchesOrBlankStore,
  };
});

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
  const MockCodeMirror = (await import('$lib/mocks/MockComponent.svelte')).default;
  return {
    ...((await importOriginal()) as object),
    codeMirrorRainlang: MockCodeMirror,
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
    handleOrderRemoveModal: vi.fn(),
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

test('calls the order detail query fn with the correct order id', async () => {
  let receivedId: string;
  mockIPC((cmd, args) => {
    if (cmd === 'order_detail') {
      receivedId = args.id as string;
      return mockOrderDetailsExtended;
    }
  });

  const queryClient = new QueryClient();

  render(OrderDetail, {
    props: { id: 'order1', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => expect(receivedId).toEqual('order1'));
});

test('shows the correct empty message when the query returns no data', async () => {
  mockIPC((cmd) => {
    if (cmd === 'order_detail') {
      return null;
    }
  });

  const queryClient = new QueryClient();

  render(OrderDetail, {
    props: { id: 'order1', network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
});

test('shows the correct data when the query returns data', async () => {
  const mockData = mockOrderDetailsExtended;
  mockIPC((cmd) => {
    if (cmd === 'order_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(OrderDetail, {
    props: { id: mockData.order.id, network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.getByText('Order')).toBeInTheDocument();
    expect(screen.getByText('0xabc...bcdef')).toBeInTheDocument();
    expect(screen.getByText('Owner')).toBeInTheDocument();
    expect(screen.getByText('0x1111111111111111111111111111111111111111')).toBeInTheDocument();
    expect(screen.getByText('Created')).toBeInTheDocument();
    expect(
      screen.getByText(formatTimestampSecondsAsLocal(BigInt(mockData.order.timestampAdded))),
    ).toBeInTheDocument(); // Adjust this to match your date formatting
  });
});

test('shows the correct data when the query returns data with inputs and outputs', async () => {
  const mockData = mockOrderDetailsExtended;
  mockIPC((cmd) => {
    if (cmd === 'order_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(OrderDetail, {
    props: { id: mockData.order.id, network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(async () => {
    // Check for input vaults
    for (const input of mockData.order.inputs) {
      expect(
        await screen.findByText(`${input.token.name} (${input.token.symbol})`),
      ).toBeInTheDocument();
      expect(
        await screen.findByText(
          `${formatUnits(BigInt(input.balance), parseInt(input.token.decimals || '18'))}`,
        ),
      ).toBeInTheDocument();
    }

    // Check for output vaults
    for (const output of mockData.order.outputs) {
      expect(
        await screen.findByText(`${output.token.name} (${output.token.symbol})`),
      ).toBeInTheDocument();
      expect(
        await screen.findByText(
          `${formatUnits(BigInt(output.balance), parseInt(output.token.decimals || '18'))}`,
        ),
      ).toBeInTheDocument();
    }
  });
});

test('shows remove button if owner wallet matches and order is active, opens correct modal', async () => {
  const mockData = mockOrderDetailsExtended;

  mockIPC((cmd) => {
    if (cmd === 'order_detail') {
      return mockData;
    }
  });

  const queryClient = new QueryClient();

  render(OrderDetail, {
    props: { id: mockData.order.id, network: 'mainnet' },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  await waitFor(() => {
    expect(screen.queryByText('Remove')).not.toBeInTheDocument();
  });

  mockWalletAddressMatchesOrBlankStore.set(() => true);

  await waitFor(() => {
    expect(screen.queryByText('Remove')).toBeInTheDocument();
  });

  screen.getByText('Remove').click();

  await waitFor(() => {
    expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockData.order, expect.any(Function));
  });
});
