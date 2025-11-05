<script lang="ts">
  import type { MetricCfg } from '@rainlanguage/orderbook';
  import type { TransformedPlotData } from '$lib/utils/chartData';
  export let metric: MetricCfg;
  export let data: TransformedPlotData[];

  $: metricDatum = data?.[0]?.[metric.value];
  $: metricValueText = metricDatum
    ? metric?.precision != null
      ? Number(metricDatum.value.toPrecision(metric.precision)).toString()
      : metricDatum.formatted
    : undefined;
</script>

<div class="flex h-full w-full flex-col items-center justify-between border p-4">
  <span>{metric.label}</span>
  <span class="text-2xl">
    {(metric?.['unit-prefix'] ?? '') + (metricValueText ?? '') + (metric?.['unit-suffix'] ?? '')}
  </span>

  {#if metric?.description}
    <span class="text-sm">{metric.description}</span>
  {/if}
</div>
