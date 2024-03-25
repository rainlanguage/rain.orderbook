<script lang="ts">
  import { html } from "htl";
  import type { PlotData } from "$lib/typeshare/fuzz";
  import * as Plot from "@observablehq/plot";
  import { hexToBigInt, type Hex, formatUnits } from "viem";
  import ModalViewRainlang from "./ModalViewRainlang.svelte";

  export let plotData: PlotData;
  let data: number[][];
  let openRainlangView = false;

  var openModal = function openModal() {
    openRainlangView = true;
  }

  $: data = plotData?.data.map(
      (row) => row.data.map(
          (val) => {
          return +formatUnits(hexToBigInt(val as Hex), 18)
          }
      )
  );

  $: rainlangText = plotData?.data[0].rainlang;

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
      ],
      subtitle: html`<button onclick=${openModal}>View Generated Rainlang</button>`
    })); // add the new chart

  }
</script>

{#if data}
<div bind:this={div} role="img" class="border p-4 w-full"></div>
{/if}

<ModalViewRainlang bind:open={openRainlangView} text={rainlangText}/>