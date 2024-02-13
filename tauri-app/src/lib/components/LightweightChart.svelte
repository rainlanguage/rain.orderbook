<script lang="ts" generics="T extends keyof SeriesOptionsMap, D, O">
  // eslint-disable-next-line no-undef
  type ISeriesApiType =  ISeriesApi<T, Time, WhitespaceData<Time>, O, DeepPartial<O & SeriesOptionsCommon>>;

  import { lightweightChartsTheme } from "$lib/stores/darkMode";
  import { ButtonGroup, Spinner } from "flowbite-svelte";
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  import { createChart, type IChartApi, type UTCTimestamp, type ISeriesApi, type WhitespaceData, type Time, type DeepPartial, type SeriesOptionsCommon, type BarPrice, type SeriesOptionsMap  } from "lightweight-charts";
  import { onDestroy, onMount } from "svelte";
  import ButtonTab from '$lib/components/ButtonTab.svelte';

  export let data: {value: number, time: UTCTimestamp, color?: string}[] = [];
  export let loading = false;
  export let emptyMessage = "None found"
  export let title: string | undefined = undefined;
  export let priceSymbol: string | undefined = undefined;
  export let createSeries: (chart: IChartApi) => ISeriesApiType;

  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
  const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

  let chartElement: HTMLElement | undefined = undefined;
  let chart: IChartApi | undefined;
  let series: ISeriesApiType | undefined;
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

  function setOptions() {
    if(chart === undefined) return;

    chart.applyOptions({ ...$lightweightChartsTheme, autoSize: true, localization: { priceFormatter: (p: BarPrice) => priceSymbol ? `${p} ${priceSymbol}` : p }});
  }


  function setData() {
    if(series === undefined || data.length === 0) return;

    series.setData(data);
    setTimeScale();
  }

  function destroyChart() {
    if (chart == undefined) return;

    chart.remove();
    chart = undefined;
  }

  function setupChart() {
    if(chartElement === undefined) return;

    chart = createChart(chartElement);
    series = createSeries(chart);
    setOptions();
  }


  $: timeDelta, setTimeScale();
  $: data, setData();
  $: $lightweightChartsTheme, setOptions();

  onMount(() => {
    setupChart();
  });
  onDestroy(() => {
    destroyChart();
  });
</script>

<div class="w-full h-full relative">
  <div bind:this={chartElement} class="w-full min-h-[32rem] h-full" {...$$props}></div>

  <div class="absolute top-5 left-5 z-50 text-gray-900 dark:text-white">
    {#if title !== undefined}
      <div class="text-xl mb-2">{title}</div>
    {/if}

    {#if data.length === 0 && !loading}
      <div>{emptyMessage}</div>
    {/if}
  </div>

  <div class="absolute top-5 right-5 z-50">
    {#if loading}
      <Spinner class="mr-2 h-4 w-4" color="white" />
    {/if}
    {#if data.length > 0}
      <ButtonGroup class="bg-gray-800">
        <ButtonTab on:click={() => (timeDelta = TIME_DELTA_1_YEAR)} active={timeDelta === TIME_DELTA_1_YEAR} size="xs" class="px-2 py-1">1 Year</ButtonTab>
        <ButtonTab on:click={() => (timeDelta = TIME_DELTA_30_DAYS)} active={timeDelta === TIME_DELTA_30_DAYS} size="xs" class="px-2 py-1">30 Days</ButtonTab>
        <ButtonTab on:click={() => (timeDelta = TIME_DELTA_7_DAYS)} active={timeDelta === TIME_DELTA_7_DAYS} size="xs" class="px-2 py-1">7 Days</ButtonTab>
        <ButtonTab on:click={() => (timeDelta = TIME_DELTA_24_HOURS)} active={timeDelta === TIME_DELTA_24_HOURS} size="xs" class="px-2 py-1">24 Hours</ButtonTab>
      </ButtonGroup>
    {/if}
  </div>
</div>