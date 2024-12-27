<script lang="ts">
  import ObservableChart from '$lib/components/ObservableChart.svelte';
  import type { ChartData } from '$lib/typeshare/config';
  import { transformDataForPlot } from '$lib/utils/chartData';
  import { sortBy } from 'lodash';
  import { MetricChart } from '@rainlanguage/ui-components';
  export let chartData: ChartData;
</script>

{#if chartData}
  <div class="mt-8 flex flex-col items-center gap-y-6">
    {#each sortBy(Object.entries(chartData.charts), ['0']) as chart}
      {@const data = transformDataForPlot(chartData.scenarios_data[chart[1].scenario.name])}
      <div class="w-full">
        <div class="flex flex-col justify-center gap-y-4">
          <h2 class="text-2xl font-bold">{chart[0]}</h2>
          <div class="grid w-full grid-cols-2 gap-4">
            {#each chart[1]?.metrics || [] as metric}
              <div class="col-span-1 flex flex-col gap-y-4">
                <MetricChart {metric} {data} />
              </div>
            {/each}
            {#each chart[1]?.plots || [] as plot}
              <div class="col-span-1 flex flex-col gap-y-4">
                <ObservableChart {plot} {data} />
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/each}
  </div>
{:else}
  No scenario data
{/if}
