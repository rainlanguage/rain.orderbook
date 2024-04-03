<script lang="ts">
  import type { Metric } from '$lib/typeshare/config';
  import type { TransformedPlotData } from '$lib/utils/chartData';
  export let metric: Metric;
  export let data: TransformedPlotData[];
</script>

<div class="flex h-full w-full flex-col items-center justify-between border p-4">
  <span>{metric.label}</span>
  <span class="text-2xl"
    >{metric?.['unit-prefix'] || ''}{data[0]?.[metric.value]}{metric?.['unit-suffix'] || ''}</span
  >
  {#if metric?.description}
    <span class="text-sm">{metric.description}</span>
  {/if}
</div>
