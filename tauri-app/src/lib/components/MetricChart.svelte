<script lang="ts">
  import type { MetricCfg } from '@rainlanguage/orderbook';
  import type { TransformedPlotData } from '$lib/utils/chartData';
  export let metric: MetricCfg;
  export let data: TransformedPlotData[];

  $: metricDatum = data?.[0]?.[metric.value];
  $: metricValueWithUnits = `${metric?.['unit-prefix'] ?? ''}${metricDatum.formatted}${metric?.['unit-suffix'] ?? ''}`;
</script>

<div class="flex h-full w-full flex-col items-center justify-between border p-4">
  <span>{metric.label}</span>
  <span class="block w-full truncate text-center text-2xl" title={metricValueWithUnits}>
    {metricValueWithUnits ?? ''}
  </span>

  {#if metric?.description}
    <span class="text-sm">{metric.description}</span>
  {/if}
</div>
