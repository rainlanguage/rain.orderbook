<script lang="ts">
  import { lightweightChartsTheme } from "$lib/stores/darkMode";
  import { ButtonGroup } from "flowbite-svelte";
  import { createChart, type IChartApi, type UTCTimestamp } from "lightweight-charts";
  import { onMount } from "svelte";
  import { v4 } from "uuid";
  import ButtonTab from '$lib/components/ButtonTab.svelte';

  export let data: {value: number, time: UTCTimestamp, color?: string}[] = [];

  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
  const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;

  let elementId: string = v4();
  let chart: IChartApi | undefined;
  let timeDelta: number = TIME_DELTA_7_DAYS;
  let timeFrom: UTCTimestamp;
  let timeTo: UTCTimestamp;

  $: {
    if(chart !== undefined) {
      timeTo = Math.floor(new Date().getTime() / 1000) as UTCTimestamp;
      timeFrom = timeTo - timeDelta as UTCTimestamp;
      chart.timeScale().setVisibleRange({
        from: timeFrom,
        to: timeTo
      });
    }
  }

  function renderChart() {
    chart = createChart(document.getElementById(elementId) as HTMLElement, { layout: $lightweightChartsTheme, autoSize: true, });
    const histogramSeries = chart.addHistogramSeries();
    histogramSeries.setData(data);
  }
  onMount(() => {
    renderChart();
  });
</script>

<div class="w-full h-full relative">
  <div id={elementId} class="w-full min-h-[32rem] h-full" {...$$props}></div>
  <div class="absolute top-5 right-5 z-50">
    <ButtonGroup class="bg-gray-800">
      <ButtonTab on:click={() => (timeDelta = TIME_DELTA_1_YEAR)} active={timeDelta === TIME_DELTA_1_YEAR} size="xs" class="px-2 py-1">1 Year</ButtonTab>
      <ButtonTab on:click={() => (timeDelta = TIME_DELTA_30_DAYS)} active={timeDelta === TIME_DELTA_30_DAYS} size="xs" class="px-2 py-1">30 Days</ButtonTab>
      <ButtonTab on:click={() => (timeDelta = TIME_DELTA_7_DAYS)} active={timeDelta === TIME_DELTA_7_DAYS} size="xs" class="px-2 py-1">7 Days</ButtonTab>
      <ButtonTab on:click={() => (timeDelta = TIME_DELTA_24_HOURS)} active={timeDelta === TIME_DELTA_24_HOURS} size="xs" class="px-2 py-1">24 Hours</ButtonTab>
    </ButtonGroup>
  </div>
</div>