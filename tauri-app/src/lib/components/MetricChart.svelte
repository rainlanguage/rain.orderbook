<script lang="ts">
  import { onDestroy } from 'svelte';
  import { writeText } from '@tauri-apps/api/clipboard';
  import ClipboardOutline from 'flowbite-svelte-icons/ClipboardOutline.svelte';
  import type { MetricCfg } from '@rainlanguage/orderbook';
  import type { TransformedPlotData } from '$lib/utils/chartData';
  export let metric: MetricCfg;
  export let data: TransformedPlotData[];

  let tooltipVisible = false;
  let copied = false;
  let copyTimeout: ReturnType<typeof setTimeout> | undefined;

  $: metricDatum = data?.[0]?.[metric.value];
  $: metricValueWithUnits = `${metric?.['unit-prefix'] ?? ''}${metricDatum.formatted}${metric?.['unit-suffix'] ?? ''}`;
  $: tooltipMessage = copied ? 'Copied!' : 'Click to copy value';

  function showTooltip() {
    tooltipVisible = true;
  }

  function hideTooltip() {
    tooltipVisible = false;
    if (copyTimeout) {
      clearTimeout(copyTimeout);
      copyTimeout = undefined;
    }
    copied = false;
  }

  async function copyMetricValue() {
    if (!metricValueWithUnits) {
      return;
    }

    try {
      await writeText(metricValueWithUnits);
    } catch (error) {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(metricValueWithUnits);
      } else {
        console.error('Unable to copy metric value', error);
        return;
      }
    }

    copied = true;
    showTooltip();
    if (copyTimeout) {
      clearTimeout(copyTimeout);
    }
    copyTimeout = setTimeout(() => {
      copied = false;
      copyTimeout = undefined;
    }, 1500);
  }

  onDestroy(() => {
    if (copyTimeout) {
      clearTimeout(copyTimeout);
    }
  });
</script>

<div class="flex h-full w-full flex-col items-center justify-between border p-4">
  <span>{metric.label}</span>
  <div class="relative w-full">
    <button
      type="button"
      class="relative flex w-full items-center justify-center gap-2 rounded bg-transparent px-2 py-1 text-2xl transition hover:bg-slate-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-500 dark:hover:bg-slate-800"
      aria-label={`Copy ${metric.label} value`}
      title={metricValueWithUnits}
      on:click={copyMetricValue}
      on:mouseenter={showTooltip}
      on:mouseleave={hideTooltip}
      on:focus={showTooltip}
      on:blur={hideTooltip}
    >
      <span class="block w-full truncate pr-6 text-center">{metricValueWithUnits ?? ''}</span>
      <ClipboardOutline
        class="absolute right-2 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-500 dark:text-slate-300"
        aria-hidden="true"
      />
    </button>

    {#if tooltipVisible}
      <div
        class="absolute bottom-full left-1/2 mb-2 -translate-x-1/2 whitespace-nowrap rounded bg-slate-900 px-2 py-1 text-xs text-white shadow-lg"
        role="status"
      >
        {tooltipMessage}
      </div>
    {/if}
  </div>

  {#if metric?.description}
    <span class="text-sm">{metric.description}</span>
  {/if}
</div>
