import { act, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '../lib/test/matchers';
import LightweightChart from '../lib/components/charts/LightweightChart.svelte';
import { type IChartApi, type UTCTimestamp } from 'lightweight-charts';
import { lightweightChartsTheme } from '../lib/stores/darkMode';
const setDataMock = vi.fn();
const applyOptionsMock = vi.fn();
const setVisibleRangeMock = vi.fn();
const removeMock = vi.fn();

vi.mock('lightweight-charts', async (importOriginal) => ({
	...((await importOriginal()) as object),
	createChart: vi.fn(() => ({
		addLineSeries: vi.fn(() => ({
			setData: setDataMock
		})),
		remove: removeMock,
		applyOptions: applyOptionsMock,
		timeScale: vi.fn(() => ({
			setVisibleRange: setVisibleRangeMock
		}))
	}))
}));

vi.mock('../lib/stores/darkMode', async (importOriginal) => {
	const { readable } = await import('svelte/store');
	return {
		...((await importOriginal()) as object),
		lightweightChartsTheme: readable({ test: 'test' })
	};
});

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
		lightweightChartsTheme
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

	const createSeries = (chart: IChartApi) => chart.addLineSeries();

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries,
		data,
		lightweightChartsTheme
	});

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartTitle')).toHaveTextContent(title);
		expect(screen.getByTestId('lightweightChartSpinner')).toBeInTheDocument();
		expect(screen.getByTestId('lightweightChartElement')).toBeInTheDocument();
		expect(screen.queryByTestId('lightweightChartYearButtons')).toBeInTheDocument();

		expect(setDataMock).toHaveBeenCalledWith(data);
	});
});

test('updates data correctly when props change', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';
	const createSeries = (chart: IChartApi) => chart.addLineSeries();

	const initialData: { value: number; time: UTCTimestamp; color?: string }[] = [
		{ value: 10, time: 1529899200 as UTCTimestamp },
		{ value: 20, time: 1529899300 as UTCTimestamp }
	];

	const newData: { value: number; time: UTCTimestamp; color?: string }[] = [
		{ value: 15, time: 1529900000 as UTCTimestamp },
		{ value: 25, time: 1529900300 as UTCTimestamp }
	];

	const { component } = render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries,
		data: initialData,
		lightweightChartsTheme
	});

	await waitFor(() => {
		expect(setDataMock).toHaveBeenCalledWith(initialData);
	});

	// Update data prop
	await act(() => component.$set({ data: newData }));

	await waitFor(() => {
		expect(setDataMock).toHaveBeenCalledWith(newData);
	});
});

test('setOptions is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		lightweightChartsTheme
	});

	await waitFor(() => {
		expect(applyOptionsMock).toHaveBeenCalled();

		const callArgs = applyOptionsMock.mock.calls[0][0];
		expect(callArgs).toMatchObject({
			test: 'test'
		});
	});
});

test('setTimeScale is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		data,
		lightweightChartsTheme
	});

	// Simulate clicking the "30 Days" button to change the timeDelta
	await waitFor(async () => {
		const timeDeltaButton = screen.getByText('30 Days');
		await act(() => fireEvent.click(timeDeltaButton));
	});

	const timeDelta = 60 * 60 * 24 * 30; // 30 days in seconds
	const timeTo = Math.floor(new Date().getTime() / 1000) as UTCTimestamp;
	const timeFrom = (timeTo - timeDelta) as UTCTimestamp;

	await waitFor(() => {
		expect(setVisibleRangeMock).toHaveBeenCalledWith({
			from: timeFrom,
			to: timeTo
		});
	});
});

test('setupChart is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const data: { value: number; time: UTCTimestamp; color?: string }[] = [
		{
			value: 10,
			time: 1529899200 as UTCTimestamp
		},
		{
			value: 20,
			time: 1529899300 as UTCTimestamp
		},
		{
			value: 5,
			time: 1529899500 as UTCTimestamp
		}
	];

	render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		data,
		lightweightChartsTheme
	});

	await waitFor(() => {
		expect(screen.getByTestId('lightweightChartElement')).toBeInTheDocument();
		expect(setDataMock).toHaveBeenCalledWith(data);
	});
});

test('destroyChart is called correctly', async () => {
	const title = 'test title';
	const emptyMessage = 'empty message';
	const loading = false;
	const priceSymbol = '$';

	const { component } = render(LightweightChart, {
		title,
		emptyMessage,
		loading,
		priceSymbol,
		createSeries: (chart: IChartApi) => chart.addLineSeries(),
		lightweightChartsTheme
	});

	component.$destroy();

	await waitFor(() => {
		expect(removeMock).toHaveBeenCalled();
	});
});
