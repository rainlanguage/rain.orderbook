import { act, render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import LightweightChart from './LightweightChart.svelte';
import type { IChartApi, UTCTimestamp } from 'lightweight-charts';

vi.mock('lightweight-charts', async (importOriginal) => ({
  ...((await importOriginal()) as object),
  createChart: vi.fn(() => ({
    addLineSeries: vi.fn(),
    remove(): void {},
    applyOptions: vi.fn(),
    timeScale: vi.fn(() => ({
      setVisibleRange: vi.fn(),
    })),
  })),
}));

test('renders without data correctly', async () => {
  const title = 'test title';
  const emptyMessage = 'empty message';
  let loading = true;
  const priceSymbol = '$';
  const createSeries = (chart: IChartApi) => chart.addLineSeries();

  const { component } = render(LightweightChart, {
    title,
    emptyMessage,
    loading,
    priceSymbol,
    createSeries,
  });

  await waitFor(() => {
    expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
    expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
  });

  loading = false;

  await act(() => component.$set({ loading: false }));

  await waitFor(() => {
    expect(screen.getByTestId('lightweightChartEmptyMessage')).toHaveTextContent(emptyMessage);
  });
});

test('renders with data correctly', async () => {
  const title = 'test title';
  const emptyMessage = 'empty message';
  const loading = true;
  const priceSymbol = '$';
  let series = { setData: vi.fn() };
  const createSeries = (chart: IChartApi) => {
    return series;
  };

  const data: { value: number; time: UTCTimestamp; color?: string }[] = [
    {
      value: 10,
      time: 1529899200 as UTCTimestamp,
    },
    {
      value: 20,
      time: 1529899300 as UTCTimestamp,
    },
    {
      value: 5,
      time: 1529899500 as UTCTimestamp,
    },
  ];

  render(LightweightChart, {
    title,
    emptyMessage,
    loading,
    priceSymbol,
    createSeries,
    data,
  });

  await waitFor(() => {
    expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
    expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
    expect(screen.getByTestId('lightweightChartElement')).toBeInTheDocument();
    expect(screen.queryByTestId('lightweightChartYearButtons')).toBeInTheDocument();

    expect();
  });
});
