<script lang="ts">
  import ObservableChart from '$lib/components/ObservableChart.svelte';
  import type { ChartData } from '$lib/typeshare/config';
  export let chartData: ChartData;
</script>

{#if chartData}
  <div class="flex flex-col items-center">
    {#each Object.entries(chartData.charts) as chart}
      <div class="w-full">
        <div class="flex flex-col justify-center gap-y-4">
          <h2 class="text-2xl font-bold">{chart[0]}</h2>
          <div class="grid w-full grid-cols-2 gap-4">
            {#each chart[1].plots as plot}
              <div class="col-span-1 flex flex-col gap-y-4">
                <ObservableChart
                  {plot}
                  scenarioData={chartData.scenarios_data[chart[1].scenario.name]}
                />
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}
