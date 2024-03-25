<script lang="ts">
  import type { PlotData } from "$lib/typeshare/fuzz";
  import * as Plot from "@observablehq/plot";
  import { hexToBigInt, type Hex, formatUnits } from "viem";

  export let plotData: PlotData;
  let data: number[][];

  $: data = plotData?.data.map(
    (row) => row.map(
      (val) => {
        return +formatUnits(hexToBigInt(val as Hex), 18)
      }
    )
);

  let div: HTMLDivElement;

  $: {
    div?.firstChild?.remove(); // remove old chart, if any
    div?.append(Plot.plot({
      title: plotData.name,
      y: {grid: true},
      x: {grid: true},
      marginLeft: 70,
      marks: [
        plotData.plot_type == "line"
          ? Plot.line(data, {x: "0", y: "1", sort: {channel: "x"}})
          : Plot.dot(data, {x: "0", y: "1", sort: {channel: "x"}}),
      ]
    })); // add the new chart

  }
</script>

{#if data}
  <div bind:this={div} role="img" class="border p-4 w-full"></div>
{/if}