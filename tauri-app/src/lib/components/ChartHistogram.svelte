<script lang="ts">
  import { lightweightChartsTheme } from "$lib/stores/darkMode";
  import { ButtonGroup, Spinner } from "flowbite-svelte";
  import { createChart, type IChartApi, type UTCTimestamp, type ISeriesApi, type HistogramData, type HistogramSeriesOptions, type HistogramStyleOptions, type WhitespaceData, type Time, type DeepPartial, type SeriesOptionsCommon,  } from "lightweight-charts";
  import { onMount } from "svelte";
  import ButtonTab from '$lib/components/ButtonTab.svelte';

  export let data: {value: number, time: UTCTimestamp, color?: string}[] = [];
  export let loading = false;
  export let emptyMessage = "None found"

  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
  const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

  let chartElement: HTMLElement | undefined = undefined;
  let chart: IChartApi | undefined;
  let series: ISeriesApi<"Histogram", Time, WhitespaceData<Time> | HistogramData<Time>, HistogramSeriesOptions, DeepPartial<HistogramStyleOptions & SeriesOptionsCommon>> | undefined;
  let timeDelta: number = TIME_DELTA_7_DAYS;
  let timeFrom: UTCTimestamp;
  let timeTo: UTCTimestamp;

  function setTimeScale() {
    if(chart === undefined) return;

    timeTo = Math.floor(new Date().getTime() / 1000) as UTCTimestamp;
      timeFrom = timeTo - timeDelta as UTCTimestamp;
      chart.timeScale().setVisibleRange({
        from: timeFrom,
        to: timeTo
      });
  }

  function setData() {
    if(series === undefined || data.length === 0) return;

    series.setData(data);
    setTimeScale();
  }

  function setOptions() {
    if(chart === undefined) return;

    chart.applyOptions({ ...$lightweightChartsTheme, autoSize: true });
  }

  function setupChart() {
    if(chartElement === undefined) return;

    chart = createChart(chartElement);
    series = chart.addHistogramSeries();
    setOptions();
  }

  $: timeDelta, setTimeScale();
  $: data, setData();
  $: $lightweightChartsTheme, setOptions();

  onMount(() => {
    setupChart();
  });
</script>

<div class="w-full h-full relative">
  <div bind:this={chartElement} class="w-full min-h-[32rem] h-full" {...$$props}></div>
  {#if data.length > 0 || loading}
    <div class="absolute top-5 right-5 z-50">
      {#if loading}
        <Spinner class="mr-2 h-4 w-4" color="white" />
      {/if}
        <ButtonGroup class="bg-gray-800">
          <ButtonTab on:click={() => (timeDelta = TIME_DELTA_1_YEAR)} active={timeDelta === TIME_DELTA_1_YEAR} size="xs" class="px-2 py-1">1 Year</ButtonTab>
          <ButtonTab on:click={() => (timeDelta = TIME_DELTA_30_DAYS)} active={timeDelta === TIME_DELTA_30_DAYS} size="xs" class="px-2 py-1">30 Days</ButtonTab>
          <ButtonTab on:click={() => (timeDelta = TIME_DELTA_7_DAYS)} active={timeDelta === TIME_DELTA_7_DAYS} size="xs" class="px-2 py-1">7 Days</ButtonTab>
          <ButtonTab on:click={() => (timeDelta = TIME_DELTA_24_HOURS)} active={timeDelta === TIME_DELTA_24_HOURS} size="xs" class="px-2 py-1">24 Hours</ButtonTab>
        </ButtonGroup>
    </div>
  {:else}
    <div class="absolute top-5 z-50 w-full text-center text-gray-900 dark:text-white">{emptyMessage}</div>
  {/if}
</div>