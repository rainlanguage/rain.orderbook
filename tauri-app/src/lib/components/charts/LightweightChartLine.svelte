<script lang="ts">
  import { LightweightChart } from '@rainlanguage/ui-components';
  import { lightweightChartsTheme } from '$lib/stores/darkMode';
</script>

<LightweightChart
  createSeries={(chart) => chart.addLineSeries({ lineWidth: 1 })}
  {...$$props}
  {lightweightChartsTheme}
/>
