<script lang="ts">
  import { lightweightChartsTheme } from "$lib/stores/darkMode";
  import { ButtonGroup, Spinner } from "flowbite-svelte";
  import { createChart, type IChartApi, type UTCTimestamp, type ISeriesApi, type HistogramData, type HistogramSeriesOptions, type HistogramStyleOptions, type WhitespaceData, type Time, type DeepPartial, type SeriesOptionsCommon,  } from "lightweight-charts";
  import { onMount } from "svelte";
  import { v4 } from "uuid";
  import ButtonTab from '$lib/components/ButtonTab.svelte';

  export let data: {value: number, time: UTCTimestamp, color?: string}[] = [];
  export let loading = false;
  export let emptyMessage = "None found"

  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
  const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

  let elementId: string = v4();
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

  function setupChart() {
    chart = createChart(document.getElementById(elementId) as HTMLElement, { layout: $lightweightChartsTheme, autoSize: true, });
    series = chart.addHistogramSeries();
  }

  $: timeDelta, setTimeScale();
  $: data, setData();

  onMount(() => {
    setupChart();
  });
</script>

<div class="w-full h-full relative">
  <div id={elementId} class="w-full min-h-[32rem] h-full" {...$$props}></div>
  {#if data.length > 0}
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