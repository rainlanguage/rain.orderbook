<script lang="ts">
  import LightweightChart from './LightweightChart.svelte';
</script>

<LightweightChart createSeries={(chart) => chart.addLineSeries({ lineWidth: 1 })} {...$$props} />
