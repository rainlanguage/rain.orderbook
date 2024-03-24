<script lang="ts">
import type { DotOptions, FuzzResultFlat, LineOptions, Mark, Plot as PlotT, Transform } from "$lib/typeshare/config";
import * as Plot from "@observablehq/plot";
import { hexToBigInt, type Hex, formatUnits, parseUnits } from "viem";

export let plot: PlotT;
export let scenarioData: FuzzResultFlat;

type PlotData = { [key: string]: number }

let data: PlotData[];

$: console.log(data);

const transformData = (fuzzResult: FuzzResultFlat): PlotData[] => {
    return fuzzResult.data.map(row => {
        const rowObject: PlotData = {};
        fuzzResult.column_names.forEach((columnName, index) => {
            rowObject[columnName] = +formatUnits(hexToBigInt(row[index] as Hex), 18);
        });
        return rowObject;
    });
}

$: data = transformData(scenarioData);

let div: HTMLDivElement;

const buildMark = (data: PlotData[], markConfig: Mark) => {
  console.log(markConfig.options);
  switch (markConfig.type) {
    case "line":
            if (markConfig.options.transform == undefined) {
              delete markConfig.options.transform;
              return Plot.line(data, markConfig.options as Omit<LineOptions, "transform">);
              } else {
              return Plot.line(data, buildTransform(markConfig.options.transform))
            }
    case "dot":
            if (markConfig.options.transform == undefined) {
              delete markConfig.options.transform;
                return Plot.dot(data, markConfig.options as Omit<DotOptions, "transform">);
              } else {
              return Plot.dot(data, buildTransform(markConfig.options.transform))
            }
    case "recty":
            if (markConfig.options.transform == undefined) {
              delete markConfig.options.transform;
              return Plot.rectY(data, markConfig.options as Omit<Plot.RectYOptions, "transform">);
              } else {
              return Plot.rectY(data, buildTransform(markConfig.options.transform))
            }
    case "axisx":
            return Plot.axisX(data, markConfig.options as Plot.AxisXOptions);
    case "axisy":
            return Plot.axisY(data, markConfig.options as Plot.AxisYOptions);
  }
}

const buildTransform = (transform: Transform) => {
    switch (transform.type) {
        case "binx":
            return Plot.binX(transform.content.outputs, transform.content.options);
    }
}

  $: {
    div?.firstChild?.remove(); // remove old chart, if any
    div?.append(Plot.plot({
      title: plot.title,
      subtitle: plot.subtitle,
      y: {grid: true},
      x: {grid: true},
      marginLeft: 70,
      marks: plot.marks.map(mark => buildMark(data, mark))
    })); // add the new chart

  }
</script>

{#if data}
<div bind:this={div} role="img" class="border p-4 w-full"></div>
{/if}