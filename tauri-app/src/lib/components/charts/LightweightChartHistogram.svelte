<script lang="ts">
  import { LightweightChart } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
</script>

<LightweightChart
  createSeries={(chart) => chart.addHistogramSeries()}
  {...$$props}
  {lightweightChartsTheme}
/>
