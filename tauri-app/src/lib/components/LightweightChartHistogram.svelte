<script lang="ts">
  import LightweightChart from "./LightweightChart.svelte";
</script>

<LightweightChart createSeries={(chart) => chart.addHistogramSeries()} {...$$props} />