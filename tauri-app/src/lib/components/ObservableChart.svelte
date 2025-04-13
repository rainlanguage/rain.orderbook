<script lang="ts">
  import type { MarkCfg, PlotCfg as PlotT, TransformCfg } from '@rainlanguage/orderbook';
  import * as Plot from '@observablehq/plot';
  import camelcaseKeys from 'camelcase-keys';
  import type { TransformedPlotData } from '$lib/utils/chartData';

  export let plot: PlotT;
  export let data: TransformedPlotData[];

  let div: HTMLDivElement;
  let error: string;

  $: if (div && data) {
    try {
      div?.firstChild?.remove(); // remove old chart, if any
      div.append(
        Plot.plot({
          ...getPlotOptions(plot),
          marks: plot.marks.map((mark) => buildMark(data, mark)),
        }),
      );
    } catch (e) {
      if (e instanceof Error) {
        error = e.message;
      } else {
        error = String(e);
      }
    }
  }

  // Get the plot options from the plot object - removing the marks
  const getPlotOptions = (plot: PlotT) => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { marks, ...plotOptions } = plot;
    return fixKeys(plotOptions);
  };

  // Map the mark object to the plot library function
  const buildMark = (data: TransformedPlotData[], markConfig: MarkCfg) => {
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
  const buildTransform = (transform: TransformCfg) => {
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
</script>

{#if error}
  <div class="w-full border p-4 text-red-500">{error}</div>
{:else}
  <div
    bind:this={div}
    role="img"
    class="w-full border p-4 [&_h2]:text-lg [&_h2]:font-semibold [&_h3]:text-sm"
    data-testid="chart"
  ></div>
{/if}
