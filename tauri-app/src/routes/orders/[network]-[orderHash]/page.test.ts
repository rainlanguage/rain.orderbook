import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Page from './+page.svelte';
import type { NewConfig } from '@rainlanguage/orderbook';
import { EMPTY_SETTINGS } from '$lib/stores/settings';

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

  describe('Missing settings tests', () => {
    beforeEach(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      mockSettingsStore.mockSetSubscribeValue(EMPTY_SETTINGS);
    });

    it('displays error when all settings are missing', () => {
      render(Page);

      expect(screen.getByText('Failed to load order')).toBeTruthy();
      expect(screen.getByText(/Missing the following items from settings for/)).toBeTruthy();
      expect(screen.getByText('ethereum')).toBeTruthy();

      expect(screen.getByText('RPC URLs')).toBeTruthy();
      expect(screen.getByText('Subgraph URL')).toBeTruthy();
      expect(screen.getByText('Orderbook Address')).toBeTruthy();
    });

    it('only displays actually missing items', async () => {
      // Set partial settings
      mockSettingsStore.mockSetSubscribeValue({
        orderbook: {
          ...EMPTY_SETTINGS.orderbook,
          orderbooks: {
            ethereum: {
              address: '0xabc',
            },
          },
          networks: {
            ethereum: {
              key: 'ethereum',
              rpcs: ['https://ethereum.example.com'],
            },
          },
        },
      } as unknown as NewConfig);

      render(Page);

      expect(screen.getByText('Subgraph URL')).toBeTruthy();

      expect(screen.queryByText('RPC URLs')).toBeFalsy();
      expect(screen.queryByText('Orderbook Address')).toBeFalsy();
    });

    it('has a link to settings when clicking Go to settings button', async () => {
      render(Page);

      const settingsLink = screen.getByText('Go to settings').closest('a');

      expect(settingsLink).toBeTruthy();
      expect(settingsLink?.getAttribute('href')).toBe('/settings');
    });
  });
});
