<script lang="ts">
  import type { FuzzResultFlat, Mark, Plot as PlotT, Transform } from '$lib/typeshare/config';
  import * as Plot from '@observablehq/plot';
  import { hexToBigInt, type Hex, formatUnits } from 'viem';
  import camelcaseKeys from 'camelcase-keys';

  export let plot: PlotT;
  export let scenarioData: FuzzResultFlat;

  type PlotData = { [key: string]: number };

  let data: PlotData[];
  $: data = transformData(scenarioData);

  // Transform the data from the backend to the format required by the plot library
  const transformData = (fuzzResult: FuzzResultFlat): PlotData[] => {
    return fuzzResult.data.map((row) => {
      const rowObject: PlotData = {};
      fuzzResult.column_names.forEach((columnName, index) => {
        rowObject[columnName] = +formatUnits(hexToBigInt(row[index] as Hex), 18);
      });
      return rowObject;
    });
  };

  // Get the plot options from the plot object - removing the marks
  const getPlotOptions = (plot: PlotT) => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { marks, ...plotOptions } = plot;
    return fixKeys(plotOptions);
  };

  // Map the mark object to the plot library function
  const buildMark = (data: PlotData[], markConfig: Mark) => {
    const options = fixKeys(markConfig.options);
    switch (markConfig.type) {
      case 'line':
        return Plot.line(
          data,
          options.transform ? buildTransform(options.transform) : { sort: options.x, ...options },
        );
      case 'dot':
        return Plot.dot(data, options.transform ? buildTransform(options.transform) : options);
      case 'recty':
        return Plot.rectY(data, options.transform ? buildTransform(options.transform) : options);
    }
  };

  // Map the transform object to the plot library function
  const buildTransform = (transform: Transform) => {
    switch (transform.type) {
      case 'binx':
        return Plot.binX(transform.content.outputs, transform.content.options);
      case 'hexbin':
        return Plot.hexbin(
          transform.content.outputs as Plot.ChannelReducers<Plot.GroupReducer>,
          transform.content.options,
        );
    }
  };

  // Delete keys with null values from an object and renaming all keys to camelCase
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const fixKeys = (obj: Record<string, any>): Record<string, any> => {
    Object.keys(obj).forEach((key) => {
      if (obj[key] && typeof obj[key] === 'object') fixKeys(obj[key]);
      else if (obj[key] == null) delete obj[key];
    });
    return camelcaseKeys(obj, { deep: true });
  };

  let div: HTMLDivElement;

  $: {
    div?.firstChild?.remove(); // remove old chart, if any
    div?.append(
      Plot.plot({
        ...getPlotOptions(plot),
        marks: plot.marks.map((mark) => buildMark(data, mark)),
      }),
    ); // add the new chart
  }
</script>

{#if data}
  <div bind:this={div} role="img" class="w-full border p-4"></div>
{/if}
