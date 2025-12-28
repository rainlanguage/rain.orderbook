<script lang="ts">
	import type { Readable } from 'svelte/store';
	import { type RaindexOrder } from '@rainlanguage/orderbook';
	import { createQuery } from '@tanstack/svelte-query';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';
	import {
		extractPairsFromTrades,
		getDefaultPair,
		getTokenLabel,
		transformPairTrades,
		TIME_DELTA_24_HOURS,
		TIME_DELTA_7_DAYS,
		TIME_DELTA_30_DAYS,
		TIME_DELTA_1_YEAR,
		type TradingPair
	} from '../../services/pairTradesChartData';
	import { Button, ButtonGroup, Dropdown, DropdownItem, Spinner } from 'flowbite-svelte';
	import { ChevronDownSolid } from 'flowbite-svelte-icons';
	import { createChart, type IChartApi, type UTCTimestamp } from 'lightweight-charts';
	import { onDestroy, onMount } from 'svelte';

	export let order: RaindexOrder;
	export let lightweightChartsTheme: Readable<Record<string, unknown>>;

	let chartElement: HTMLElement | undefined = undefined;
	let chart: IChartApi | undefined;
	let priceSeries: ReturnType<IChartApi['addLineSeries']> | undefined;
	let buyVolumeSeries: ReturnType<IChartApi['addHistogramSeries']> | undefined;
	let sellVolumeSeries: ReturnType<IChartApi['addHistogramSeries']> | undefined;

	let timeDelta = TIME_DELTA_1_YEAR;
	let pairs: TradingPair[] = [];
	let selectedPairIndex = 0;
	let dropdownOpen = false;

	$: selectedPair = pairs[selectedPairIndex] ?? null;

	$: query = createQuery({
		queryKey: [QKEY_ORDER_TRADES_LIST, order.id],
		queryFn: async () => {
			const data = await order.getTradesList(undefined, undefined, 1);
			if (data.error) throw new Error(data.error.readableMsg);
			return data.value;
		}
	});

	$: trades = $query?.data ?? [];

	$: {
		if (trades.length > 0 && pairs.length === 0) {
			pairs = extractPairsFromTrades(trades);
			const defaultPair = getDefaultPair(trades);
			if (defaultPair) {
				const idx = pairs.findIndex(
					(p) =>
						p.baseToken.address.toLowerCase() === defaultPair.baseToken.address.toLowerCase() &&
						p.quoteToken.address.toLowerCase() === defaultPair.quoteToken.address.toLowerCase()
				);
				if (idx >= 0) selectedPairIndex = idx;
			}
		}
	}

	$: chartData = selectedPair
		? transformPairTrades({
				trades,
				baseTokenAddress: selectedPair.baseToken.address,
				quoteTokenAddress: selectedPair.quoteToken.address,
				timeDeltaSeconds: timeDelta
			})
		: null;

	$: pairLabel = selectedPair
		? `${getTokenLabel(selectedPair.baseToken)}/${getTokenLabel(selectedPair.quoteToken)}`
		: 'Select pair';

	function flipPair() {
		if (selectedPair) {
			const flippedPair: TradingPair = {
				baseToken: selectedPair.quoteToken,
				quoteToken: selectedPair.baseToken
			};
			const existingIdx = pairs.findIndex(
				(p) =>
					p.baseToken.address.toLowerCase() === flippedPair.baseToken.address.toLowerCase() &&
					p.quoteToken.address.toLowerCase() === flippedPair.quoteToken.address.toLowerCase()
			);
			if (existingIdx >= 0) {
				selectedPairIndex = existingIdx;
			} else {
				pairs = [...pairs, flippedPair];
				selectedPairIndex = pairs.length - 1;
			}
		}
	}

	function selectPair(index: number) {
		selectedPairIndex = index;
		dropdownOpen = false;
	}

	function setupChart() {
		if (!chartElement) return;

		chart = createChart(chartElement, {
			autoSize: true,
			leftPriceScale: {
				visible: true,
				borderVisible: false
			},
			rightPriceScale: {
				visible: true,
				borderVisible: false
			},
			crosshair: {
				mode: 0
			}
		});

		buyVolumeSeries = chart.addHistogramSeries({
			color: 'rgba(38, 166, 154, 0.5)',
			priceFormat: { type: 'volume' },
			priceScaleId: 'left'
		});

		sellVolumeSeries = chart.addHistogramSeries({
			color: 'rgba(239, 83, 80, 0.5)',
			priceFormat: { type: 'volume' },
			priceScaleId: 'left'
		});

		priceSeries = chart.addLineSeries({
			color: '#666666',
			lineWidth: 2,
			priceScaleId: 'right',
			priceFormat: {
				type: 'price',
				precision: 6,
				minMove: 0.000001
			}
		});

		chart.priceScale('left').applyOptions({
			scaleMargins: { top: 0.1, bottom: 0.1 }
		});

		chart.priceScale('right').applyOptions({
			scaleMargins: { top: 0.1, bottom: 0.1 }
		});

		setChartOptions();
		updateChartData();
	}

	function setChartOptions() {
		if (!chart) return;
		chart.applyOptions({
			...$lightweightChartsTheme,
			autoSize: true
		});
	}

	function updateChartData() {
		if (!chart || !priceSeries || !buyVolumeSeries || !sellVolumeSeries) return;

		if (chartData?.success && chartData.data) {
			priceSeries.setData(chartData.data.pricePoints);
			buyVolumeSeries.setData(chartData.data.buyVolumePoints);
			sellVolumeSeries.setData(chartData.data.sellVolumePoints);
		} else {
			priceSeries.setData([]);
			buyVolumeSeries.setData([]);
			sellVolumeSeries.setData([]);
		}

		setTimeScale();
	}

	function setTimeScale() {
		if (!chart) return;
		const now = Math.floor(Date.now() / 1000) as UTCTimestamp;
		const from = (now - timeDelta) as UTCTimestamp;
		chart.timeScale().setVisibleRange({ from, to: now });
	}

	function destroyChart() {
		if (chart) {
			chart.remove();
			chart = undefined;
			priceSeries = undefined;
			buyVolumeSeries = undefined;
			sellVolumeSeries = undefined;
		}
	}

	$: if (chartData) updateChartData();
	$: if (timeDelta && chart) setTimeScale();
	$: if ($lightweightChartsTheme && chart) setChartOptions();
	$: if (chartElement && trades.length > 0 && selectedPair && !chart) setupChart();

	onMount(() => {
		if (trades.length > 0 && selectedPair) setupChart();
	});

	onDestroy(() => {
		destroyChart();
	});
</script>

<div
	class="flex h-full flex-col overflow-hidden rounded-lg border bg-gray-50 dark:border-none dark:bg-gray-700"
>
	<div
		class="flex w-full flex-wrap items-center justify-between gap-2 border-b p-3 dark:border-gray-700"
	>
		<div class="flex items-center gap-2">
			<div class="text-xl text-gray-900 dark:text-white" data-testid="chart-title">Trades</div>
			{#if $query?.isLoading}
				<Spinner class="h-4 w-4" color="white" data-testid="chart-spinner" />
			{/if}
		</div>

		<div class="flex flex-wrap items-center gap-2">
			{#if pairs.length > 0}
				<div class="flex items-center gap-1">
					<Button
						color="alternative"
						size="xs"
						class="flex items-center gap-1"
						data-testid="pair-selector"
					>
						{pairLabel}
						<ChevronDownSolid class="h-3 w-3" />
					</Button>
					<Dropdown bind:open={dropdownOpen} data-testid="pair-dropdown">
						{#each pairs as pair, idx}
							<DropdownItem on:click={() => selectPair(idx)} data-testid="pair-option-{idx}">
								{getTokenLabel(pair.baseToken)}/{getTokenLabel(pair.quoteToken)}
							</DropdownItem>
						{/each}
					</Dropdown>
					<Button
						color="alternative"
						size="xs"
						on:click={flipPair}
						title="Flip pair"
						data-testid="flip-button"
					>
						<svg
							class="h-4 w-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
							xmlns="http://www.w3.org/2000/svg"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
							/>
						</svg>
					</Button>
				</div>
			{/if}

			{#if trades.length > 0}
				<ButtonGroup class="bg-gray-800" data-testid="time-filters">
					<Button
						on:click={() => (timeDelta = TIME_DELTA_1_YEAR)}
						color={timeDelta === TIME_DELTA_1_YEAR ? 'primary' : 'alternative'}
						size="xs"
						class="px-2 py-1">1Y</Button
					>
					<Button
						on:click={() => (timeDelta = TIME_DELTA_30_DAYS)}
						color={timeDelta === TIME_DELTA_30_DAYS ? 'primary' : 'alternative'}
						size="xs"
						class="px-2 py-1">30D</Button
					>
					<Button
						on:click={() => (timeDelta = TIME_DELTA_7_DAYS)}
						color={timeDelta === TIME_DELTA_7_DAYS ? 'primary' : 'alternative'}
						size="xs"
						class="px-2 py-1">7D</Button
					>
					<Button
						on:click={() => (timeDelta = TIME_DELTA_24_HOURS)}
						color={timeDelta === TIME_DELTA_24_HOURS ? 'primary' : 'alternative'}
						size="xs"
						class="px-2 py-1">24H</Button
					>
				</ButtonGroup>
			{/if}
		</div>
	</div>

	<div class="relative flex w-full grow items-center justify-center bg-white dark:bg-gray-800">
		{#if $query?.isLoading}
			<div class="text-gray-800 dark:text-gray-400">Loading...</div>
		{:else if $query?.isError}
			<div class="text-red-500" data-testid="chart-error">
				Error: {$query?.error?.message ?? 'Failed to load trades'}
			</div>
		{:else if chartData && !chartData.success}
			<div class="text-red-500" data-testid="chart-transform-error">
				Error: {chartData.error}
			</div>
		{:else if trades.length === 0 || !selectedPair}
			<div class="text-gray-800 dark:text-gray-400" data-testid="chart-empty">No trades found</div>
		{:else if chartData?.success && chartData.data.pricePoints.length === 0}
			<div class="text-gray-800 dark:text-gray-400" data-testid="chart-no-data">
				No trades found for selected pair and time range
			</div>
		{:else}
			<div
				bind:this={chartElement}
				class="h-full w-full overflow-hidden"
				data-testid="chart-element"
			></div>
		{/if}
	</div>
</div>
