import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';
import type { NewConfig } from '@rainlanguage/orderbook';

const { mockPageStore, mockSettingsStore, MockComponent } = await vi.hoisted(
  () => import('@rainlanguage/ui-components'),
);

vi.mock('$app/stores', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  return {
    ...original,
    page: mockPageStore,
  };
});

vi.mock('$lib/stores/settings', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  return {
    ...original,
    settings: mockSettingsStore,
  };
});

vi.mock('$lib/utils/getOrderbookByChainId', () => ({
  getOrderbookByChainId: vi.fn().mockReturnValue({
    network: {
      rpcs: ['http://localhost:8545'],
    },
  }),
}));

vi.mock('@tanstack/svelte-query', () => ({
  useQueryClient: vi.fn().mockReturnValue({
    invalidateQueries: vi.fn(),
  }),
}));

vi.mock('$lib/services/modal', () => ({
  handleDebugTradeModal: vi.fn(),
  handleQuoteDebugModal: vi.fn(),
  handleDepositModal: vi.fn(),
  handleWithdrawModal: vi.fn(),
  handleOrderRemoveModal: vi.fn(),
}));

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
  const original = (await importOriginal()) as object;
  return {
    ...original,
    OrderDetail: MockComponent,
  };
});

describe('Order Page', () => {
  beforeAll(() => {
    mockPageStore.mockSetSubscribeValue({
      params: {
        network: 'ethereum',
        orderHash: '0x123',
      },
    });
  });

  it('renders OrderDetail when all settings are available', () => {
    mockSettingsStore.mockSetSubscribeValue({
      orderbook: {
        version: '1',
        orderbooks: {
          ethereum: {
            key: 'ethereum',
            network: {
              key: 'ethereum',
              rpcs: ['https://ethereum.example.com'],
              chainId: 1,
            },
            address: '0xabc',
            subgraph: {
              key: 'ethereum',
              url: 'https://api.thegraph.com/subgraphs/name/example',
            },
          },
        },
        subgraphs: {
          ethereum: {
            key: 'ethereum',
            url: 'https://api.thegraph.com/subgraphs/name/example',
          },
        },
        networks: {
          ethereum: {
            key: 'ethereum',
            rpcs: ['https://ethereum.example.com'],
            chainId: 1,
          },
        },
      },
    } as unknown as NewConfig);
    render(Page);

    expect(screen.getByTestId('page-header')).toBeTruthy();
    expect(screen.getByTestId('order-detail')).toBeTruthy();
  });
});
