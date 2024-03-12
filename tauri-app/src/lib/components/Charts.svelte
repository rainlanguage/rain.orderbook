<script lang="ts">
    import ObservableChart from "$lib/components/ObservableChart.svelte";
import type { ChartData } from "$lib/typeshare/fuzz";
    export let chartData: ChartData[] = [];
</script>

<div class="flex flex-col items-center">
    {#each chartData as chart}
        <div class="w-full">
            <div class="flex flex-col justify-center gap-y-4">
                <h2 class="text-2xl font-bold">{chart.name}</h2>
                <div class="grid grid-cols-2 gap-4 w-full">
                    {#each chart.plots as plotData}
                        <div class="flex flex-col gap-y-4 col-span-1">
                            <ObservableChart {plotData} />
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    {/each}
</div>