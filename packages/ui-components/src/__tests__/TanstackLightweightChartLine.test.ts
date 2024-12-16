import { render, waitFor } from '@testing-library/svelte';
import { test, expect, vi } from 'vitest';
import TanstackLightweightChartLine from '../lib/components/charts/TanstackLightweightChartLine.svelte';
import { props } from '../lib/__mocks__/MockComponent';
import { createResolvableQuery } from '../lib/__mocks__/queries';
import type { UTCTimestamp } from 'lightweight-charts';
import { get } from 'svelte/store';
import { lightweightChartsTheme } from '../lib/stores/darkMode';

// Mock the LightweightChart component
vi.mock('../lib/components/charts/LightweightChart.svelte', async () => {
	const MockLightweightChart = (await import('../lib/__mocks__/MockComponent.svelte')).default;
	return { default: MockLightweightChart };
});

type MockData = {
	value: number;
	time: number;
};

test('renders the loading state correctly', async () => {
	const mockData: MockData[] = [
		{ value: 10, time: 1629899200 },
		{ value: 20, time: 1629899300 }
	];

	const { query, resolve } = createResolvableQuery(() => mockData);

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { component: _component } = render(TanstackLightweightChartLine, {
		props: {
			query,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			timeTransform: (d: any) => d.time as UTCTimestamp,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			valueTransform: (d: any) => d.value,
			lightweightChartsTheme
		}
	});

	await waitFor(() => {
		const _props = get(props) as (typeof _component)['$$props'];
		expect(_props.loading).toBe(true);
	});

	await resolve();

	await waitFor(() => {
		const _props = get(props) as (typeof _component)['$$props'];
		expect(_props.loading).toBe(false);
		expect(_props.data).toEqual(mockData);
	});
});

test('sorts the data correctly according to time', async () => {
	const mockData: MockData[] = [
		{ value: 20, time: 1629899300 },
		{ value: 10, time: 1629899200 },
		{ value: 40, time: 1629899500 },
		{ value: 30, time: 1629899400 }
	];

	const { query, resolve } = createResolvableQuery(() => mockData);

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { component: _component } = render(TanstackLightweightChartLine, {
		props: {
			query,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			timeTransform: (d: any) => d.time as UTCTimestamp,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			valueTransform: (d: any) => d.value,
			lightweightChartsTheme
		}
	});

	await resolve();

	await waitFor(() => {
		const _props = get(props) as (typeof _component)['$$props'];
		expect(_props.data).toEqual([
			{ value: 10, time: 1629899200 },
			{ value: 20, time: 1629899300 },
			{ value: 30, time: 1629899400 },
			{ value: 40, time: 1629899500 }
		]);
	});
});

test('that a line series is added to the chart', async () => {
	const mockData: MockData[] = [{ value: 20, time: 1629899300 }];

	const { query, resolve } = createResolvableQuery(() => mockData);

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	const { component: _component } = render(TanstackLightweightChartLine, {
		props: {
			query,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			timeTransform: (d: any) => d.time as UTCTimestamp,
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			valueTransform: (d: any) => d.value,
			lightweightChartsTheme
		}
	});

	await resolve();

	await waitFor(() => {
		const _props = get(props) as (typeof _component)['$$props'];
		// Assert that the createSeries function matches the expected implementation
		expect(_props.createSeries.toString()).toEqual(
			'chart => chart.addLineSeries({ lineWidth: 1 })'
		);
	});
});
