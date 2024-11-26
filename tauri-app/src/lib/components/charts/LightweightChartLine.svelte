<script lang="ts">
  import { LightweightChart } from '@rainlanguage/ui-components';
</script>

<LightweightChart createSeries={(chart) => chart.addLineSeries({ lineWidth: 1 })} {...$$props} />
