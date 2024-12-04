/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi, type Mock } from 'vitest';
import { expect } from '../lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import OrderDetail from '../lib/components/detail/OrderDetail.svelte';
import { formatTimestampSecondsAsLocal } from '../lib/utils/time';
import type { Order } from '@rainlanguage/orderbook/js_api';
import { readable, writable } from 'svelte/store';
import { componentProps } from 'svelte'; // Import componentProps from svelte


const mockOrder: Order = {
  id: '0xabc...bcdef',
  owner: '0x1111111111111111111111111111111111111111',
  meta: '0x',
  timestampAdded: '1234567890',
  orderHash: '0xabcdef1234567890',
  expression: '0x',
  interpreter: '0x',
  dispatch: '0x',
  active: true,
  orderbook: {id: '1'},
  inputs: [
    {
      token: {
        id: '1',
        address: '0x1234567890abcdef',
        name: 'Token A',
        symbol: 'TKA',
        decimals: '18'
      },
      balance: '1000000000000000000',
      vaultId: '1'
    }
  ],
  outputs: [
    {
      token: {
        id: '2',
        address: '0xfedcba0987654321',
        name: 'Token B',
        symbol: 'TKB',
        decimals: '18'
      },
      balance: '2000000000000000000',
      vaultId: '2'
    }
  ]
};

vi.mock('@tanstack/svelte-query');

vi.mock('../lib/components/CodeMirrorRainlang.svelte', async (importOriginal) => {
  const MockCodeMirror = (await import('../lib/__mocks__/MockComponent.svelte')).default;
  return {
    default: MockCodeMirror
  };
});

vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async (importOriginal) => {
  const MockChartLine = (await import('../lib/__mocks__/MockComponent.svelte')).default;
  return {
    default: MockChartLine
  };
});

vi.mock('../lib/components/detail/TanstackOrderQuote.svelte', async (importOriginal) => {
  const MockComponent = (await import('../lib/__mocks__/MockComponent.svelte')).default;
  const { props } = await import('../lib/__mocks__/MockComponent');

  return {
    default: class extends MockComponent {
      static getProps() {
        let currentProps: unknown;
        props.subscribe(value => {
          currentProps = value;
        })();
        return currentProps;
      }
    }
  };
});

vi.mock('lightweight-charts', async (importOriginal) => ({
  ...((await importOriginal()) as object),
  createChart: vi.fn(() => ({
    addLineSeries: vi.fn(),
    remove(): void {},
    applyOptions: vi.fn(),
  })),
}));

describe('OrderDetail Component', () => {
  it('shows the correct empty message when the query returns no data', async () => {
    const queryClient = new QueryClient();
    const codeMirrorTheme = writable({});

    const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
    mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      subscribe: (fn: (value: any) => void) => {
        fn({
          data: null,
          status: 'success',
          isFetching: false
        });
        return { unsubscribe: () => {} };
      }
    })) as Mock;

    render(OrderDetail, {
      props: {
        id: 'order1',
        rpcUrl: 'https://example.com',
        subgraphUrl: 'https://example.com',
        colorTheme: writable('light'),
        codeMirrorTheme,
        lightweightChartsTheme: {}
      },
      context: new Map([['$$_queryClient', queryClient]]),
    });

    await waitFor(() => expect(screen.getByText('Order not found')).toBeInTheDocument());
  });

  it('shows remove button if owner wallet matches and order is active, opens correct modal', async () => {
    const queryClient = new QueryClient();
    const handleOrderRemoveModal = vi.fn();
    const codeMirrorTheme = writable({});

    const mockQuery = vi.mocked(await import('@tanstack/svelte-query'));
    mockQuery.createQuery = vi.fn((__options, _queryClient) => ({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      subscribe: (fn: (value: any) => void) => {
        fn({
          data: mockOrder,
          status: 'success',
          isFetching: false,
          refetch: () => {}
        });
        return { unsubscribe: () => {} };
      }
    })) as Mock;

    const { rerender } = render(OrderDetail, {
      props: {
        id: mockOrder.id,
        rpcUrl: 'https://example.com',
        subgraphUrl: 'https://example.com',
        colorTheme: writable('light'),
        codeMirrorTheme,
        lightweightChartsTheme: {},
        walletAddressMatchesOrBlank: readable(true),
        handleOrderRemoveModal
      },
      context: new Map([['$$_queryClient', queryClient]]),
    });

    await waitFor(() => {
      expect(screen.getByTestId('remove-button')).not.toBeInTheDocument();
    });

    rerender({
      id: mockOrder.id,
      rpcUrl: 'https://example.com',
      subgraphUrl: 'https://example.com',
      colorTheme: writable('light'),
      codeMirrorTheme,
      lightweightChartsTheme: {},
      walletAddressMatchesOrBlank: readable(true),
      handleOrderRemoveModal
    });

    await waitFor(() => {
      expect(screen.queryByText('Remove')).toBeInTheDocument();
    });

    screen.getByText('Remove').click();

    await waitFor(() => {
      expect(handleOrderRemoveModal).toHaveBeenCalledWith(mockOrder, expect.any(Function));
    });
  });
});
