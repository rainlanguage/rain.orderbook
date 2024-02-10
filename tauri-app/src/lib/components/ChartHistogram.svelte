<script lang="ts">
  import { ColorType, createChart } from "lightweight-charts";
    import { onMount } from "svelte";
  import { v4 } from "uuid";

  export let data: {value: number, time: string, color?: string}[] = [];
  let elementId: string = v4();

  function renderChart() {
    const chartOptions = { layout: { textColor: 'black', background: { type: ColorType.Solid, color: 'white' } } };
    const chart = createChart(document.getElementById(elementId) as HTMLElement, chartOptions);
    const histogramSeries = chart.addHistogramSeries({ color: '#26a69a' });
    histogramSeries.setData(data);
    chart.timeScale().fitContent();
  }
  onMount(() => {
    renderChart();
  });
</script>

<div id={elementId} class="w-full h-72"></div>