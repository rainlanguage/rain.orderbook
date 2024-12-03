<script lang="ts" generics="T extends keyof SeriesOptionsMap, D, O">
	import type { Readable } from 'svelte/store';

	// eslint-disable-next-line no-undef
	type ISeriesApiType = ISeriesApi<
		// eslint-disable-next-line no-undef
		T,
		Time,
		WhitespaceData<Time>,
		// eslint-disable-next-line no-undef
		O,
		// eslint-disable-next-line no-undef
		DeepPartial<O & SeriesOptionsCommon>
	>;

	import { Spinner } from 'flowbite-svelte';
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	import {
		createChart,
		type IChartApi,
		type UTCTimestamp,
		type ISeriesApi,
		type WhitespaceData,
		type Time,
		type DeepPartial,
		type SeriesOptionsCommon,
		type BarPrice,
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		type SeriesOptionsMap
	} from 'lightweight-charts';
	import { onDestroy, onMount } from 'svelte';
	import ChartTimeFilters from './ChartTimeFilters.svelte';

	export let data: { value: number; time: UTCTimestamp; color?: string }[] = [];
	export let loading = false;
	export let emptyMessage = 'None found';
	export let title: string | undefined = undefined;
	export let priceSymbol: string | undefined = undefined;
	export let createSeries: (chart: IChartApi) => ISeriesApiType;
	export let lightweightChartsTheme: Readable<Record<string, unknown>>;

	let chartElement: HTMLElement | undefined = undefined;
	let chart: IChartApi | undefined;
	let series: ISeriesApiType | undefined;
	let timeDelta: number;
	let timeFrom: UTCTimestamp;
	let timeTo: UTCTimestamp;

	function setTimeScale() {
		if (chart === undefined) return;

		timeTo = Math.floor(new Date().getTime() / 1000) as UTCTimestamp;
		timeFrom = (timeTo - timeDelta) as UTCTimestamp;
		chart.timeScale().setVisibleRange({
			from: timeFrom,
			to: timeTo
		});
	}

	function setOptions() {
		if (chart === undefined) return;

		chart.applyOptions({
			...$lightweightChartsTheme,
			autoSize: true,
			localization: {
				priceFormatter: (p: BarPrice) =>
					priceSymbol ? `${p.toFixed(4)} ${priceSymbol}` : p.toFixed(4)
			}
		});
	}

	function setData() {
		if (series === undefined || data.length === 0) return;
		series.setData(data);
		setTimeScale();
	}

	function destroyChart() {
		if (chart == undefined) return;

		chart.remove();
		chart = undefined;
	}

	function setupChart() {
		if (chartElement === undefined) return;

		chart = createChart(chartElement);
		series = createSeries(chart);
		setOptions();
	}

	$: if (data || series) setData();
	$: if (timeDelta) setTimeScale();
	$: if ($lightweightChartsTheme) setOptions();

	onMount(() => {
		setupChart();
	});
	onDestroy(() => {
		destroyChart();
	});
</script>

<div
	class="flex h-full flex-col overflow-hidden rounded-lg border bg-gray-50 dark:border-none dark:bg-gray-700"
>
	<div class="flex w-full justify-between border-b p-3 pb-0 dark:border-gray-700">
		<div class="text-gray-900 dark:text-white">
			{#if title !== undefined}
				<div data-testid="lightweightChartTitle" class="mb-2 text-xl">{title}</div>
			{/if}
		</div>

		<div>
			{#if loading}
				<Spinner data-testid="lightweightChartSpinner" class="mr-2 h-4 w-4" color="white" />
			{/if}
			{#if data.length > 0}
				<ChartTimeFilters bind:timeDelta />
			{/if}
		</div>
	</div>
	<div class="relative flex w-full grow items-center justify-center bg-white dark:bg-gray-800">
		{#if data.length === 0 && !loading}
			<div class="text-gray-800 dark:text-gray-400" data-testid="lightweightChartEmptyMessage">
				{emptyMessage}
			</div>
		{:else}
			<div
				bind:this={chartElement}
				class="h-full w-full overflow-hidden"
				{...$$props}
				data-testid="lightweightChartElement"
			></div>
		{/if}
	</div>
</div>
