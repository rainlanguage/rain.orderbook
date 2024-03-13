<script lang="ts">
import type { PlotData } from "$lib/typeshare/fuzz";
import * as Plot from "@observablehq/plot";
import { hexToBigInt, type Hex } from "viem";

export let plotData: PlotData;
let data: bigint[][];

$: data = plotData?.data.map(
    (row) => row.map(
        (val) => {
         return hexToBigInt(val as Hex)
        }
    )
);

let div: HTMLDivElement;

  $: {
    div?.firstChild?.remove(); // remove old chart, if any
    // div?.append(Plot.line(data, {x: "0", y: "1"}).plot()); // add the new chart
    div?.append(Plot.plot({
      y: {grid: true},
      x: {grid: true},
      marks: [
        plotData.plot_type == "line" ?
        Plot.line(data, {x: "0", y: "1", sort: {channel: "x"}}):
        Plot.dot(data, {x: "0", y: "1", sort: {channel: "x"}}),
      ]
    })); // add the new chart

  }
</script>

{#if data}
<div>{plotData.name}</div>
<div bind:this={div} role="img" class="border p-4"></div>
{/if}