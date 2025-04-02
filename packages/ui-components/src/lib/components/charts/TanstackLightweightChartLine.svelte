<script lang="ts" generics="T">
	import type { Readable } from 'svelte/store';
	import LightweightChart from './LightweightChart.svelte';
	import type { CreateQueryResult } from '@tanstack/svelte-query';
	import type { IChartApi, UTCTimestamp } from 'lightweight-charts';
	import { transformAndSortData } from './transformAndSortData';

	export let query: CreateQueryResult<T[]>;
	export let timeTransform: (data: T) => UTCTimestamp;
	export let valueTransform: (data: T) => number;
	export let lightweightChartsTheme: Readable<Record<string, unknown>>;

	$: data = transformAndSortData($query.data ?? [], {
		valueTransform,
		timeTransform
	});

	const createSeries = (chart: IChartApi) => chart.addLineSeries({ lineWidth: 1 });
</script>

<LightweightChart
	{createSeries}
	{data}
	loading={$query.isLoading}
	{...$$props}
	{lightweightChartsTheme}
/>
