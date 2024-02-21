<script lang="ts">
  import LightweightChart from './LightweightChart.svelte';
</script>

<LightweightChart
  createSeries={(chart) => chart.addLineSeries({ lineWidth: '0.5' })}
  {...$$props}
/>
