import { render } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import { QueryClient } from '@tanstack/svelte-query';
import VaultBalanceChart from '../lib/components/charts/VaultBalanceChart.svelte';
import type { Vault } from '../lib/typeshare/subgraphTypes';
import { getVaultBalanceChanges } from '@rainlanguage/orderbook/js_api';
import { writable } from 'svelte/store';

vi.mock('@rainlanguage/orderbook/js_api', () => ({
  getVaultBalanceChanges: vi.fn()
}));

vi.mock('../lib/components/charts/TanstackLightweightChartLine.svelte', async () => {
	const MockLightweightChart = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockLightweightChart };
});


const mockVault: Vault = {
  id: 'vault1',
  vaultId: 'vault1',
  token: {
    id: 'token1',
    address: '0xTokenAddress1',
    name: 'Token1',
    symbol: 'TKN1',
    decimals: '18',
  },
  owner: '0xOwnerAddress',
  ordersAsInput: [],
  ordersAsOutput: [],
  balanceChanges: [],
  balance: '1000000000000000000',
  orderbook: {
    id: '0x00',
  },
};

test('calls getVaultBalanceChanges with correct arguments', async () => {
  const queryClient = new QueryClient();

  render(VaultBalanceChart, {
    props: {
      vault: mockVault,
      subgraphUrl: 'https://example.com',
      lightweightChartsTheme: writable({})
    },
    context: new Map([['$$_queryClient', queryClient]]),
  });

  expect(getVaultBalanceChanges).toHaveBeenCalledWith(
    'https://example.com',
    'vault1',
    {
      first: 1000,
      skip: 0
    }
  );
});
