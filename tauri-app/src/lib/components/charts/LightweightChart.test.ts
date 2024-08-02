import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import LightweightChart from './LightweightChart.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Vault } from '$lib/typeshare/vaultDetail';
import { goto } from '$app/navigation';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
import type { IChartApi } from 'lightweight-charts';

vi.mock('lightweight-charts', async (importOriginal) => ({
  ...((await importOriginal()) as object),
  createChart: vi.fn(() => ({
    addLineSeries: vi.fn(),
    remove(): void {},
    applyOptions: vi.fn(),
  })),
}));

test('renders without data correctly', async () => {
  const title = 'test title';
  const emptyMessage = 'empty message';
  let loading = true;
  const priceSymbol = '$';
  const createSeries = (chart: IChartApi) => chart.addLineSeries();

  render(LightweightChart, { title, emptyMessage, loading, priceSymbol, createSeries });

  await waitFor(() => {
    expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
    expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
  });
});
