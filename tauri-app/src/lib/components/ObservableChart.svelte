<script lang="ts">
import type { PlotData } from "$lib/typeshare/fuzz";
import * as Plot from "@observablehq/plot";
import { hexToBigInt, type Hex, formatUnits } from "viem";
// import ModalViewRainlang from "./ModalViewRainlang.svelte";

export let plotData: PlotData;
let data: number[][];
// let openRainlangView = false;

$: data = plotData?.data.map(
    (row) => row.data.map(
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
        plotData.plot_type == "line" ?
        Plot.line(data, {x: "0", y: "1", sort: {channel: "x"}}):
        Plot.dot(data, {x: "0", y: "1", sort: {channel: "x"}}),
      ],
      // subtitle: html`<a href="#" on:click={() => openRainlangView = true}>View Generated Rainlang</a>`,
    })); // add the new chart

  }
</script>

{#if data}
<div bind:this={div} role="img" class="border p-4 w-full"></div>
{/if}

<!-- <ModalViewRainlang bind:open={openRainlangView} text={}/> -->