<script lang="ts">
import type { DotOptions, FuzzResultFlat, LineOptions, Mark, Plot as PlotT, Transform } from "$lib/typeshare/config";
import * as Plot from "@observablehq/plot";
import { hexToBigInt, type Hex, formatUnits } from "viem";

export let plot: PlotT;
export let scenarioData: FuzzResultFlat;

type PlotData = { [key: string]: number }

let data: PlotData[];

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
  switch (markConfig.type) {
    case "line":
            if (markConfig.options.transform == undefined) {
              delete markConfig.options.transform;
              return Plot.line(data, {sort: markConfig.options.x, ...markConfig.options} as Omit<LineOptions, "transform">);
              } else {
              return Plot.line(data, buildTransform(markConfig.options.transform))
            }
    case "dot":
            if (markConfig.options.transform == undefined) {
                return Plot.dot(data, markConfig.options as Omit<DotOptions, "transform">);
              } else {
              return Plot.dot(data, buildTransform(markConfig.options.transform))
            }
    case "recty":
            if (markConfig.options.transform == undefined) {
              delete markConfig.options.transform;
              return Plot.rectY(data, markConfig.options as Omit<Plot.RectYOptions, "transform">);
              } else {
                console.log('rect transform')
                console.log(markConfig.options.transform)
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
            return Plot.binX(deleteNullKeys(transform.content.outputs), deleteNullKeys(transform.content.options));
        case "hexbin":
            const options: Plot.HexbinOptions = {
              binWidth: transform.content.options?.["bin-width"],
              ...transform.content.options}
            return Plot.hexbin(deleteNullKeys(transform.content.outputs), deleteNullKeys(options));
    }
}

const deleteNullKeys = (obj: any) => {
    Object.keys(obj).forEach(key => {
        if (obj[key] && typeof obj[key] === 'object') deleteNullKeys(obj[key]);
        else if (obj[key] == null) delete obj[key];
    });
    return obj;
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