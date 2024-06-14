<script lang="ts">
  import type { Mark, Plot as PlotT, Transform } from '$lib/typeshare/config';
  import * as Plot from '@observablehq/plot';
  import camelcaseKeys from 'camelcase-keys';
  import type { TransformedPlotData } from '$lib/utils/chartData';
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  import { html } from 'htl';

  export let plot: PlotT;
  export let data: TransformedPlotData[];

  let div: HTMLDivElement;
  let error: string;

  $: if (div && data) {
    try {
      const plotData = getPlotOptions(plot);
      div?.firstChild?.remove(); // remove old chart, if any
      div.append(
        Plot.plot({
          ...plotData,
          subtitle: plotData.subtitle
            ? html`<div class="text-xs">${plotData.subtitle}</div>`
            : undefined,
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
  const buildMark = (data: TransformedPlotData[], markConfig: Mark) => {
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
</script>

{#if error}
  <div class="w-full border p-4 text-red-500">{error}</div>
{:else}
  <div bind:this={div} role="img" class="w-full border p-4"></div>
{/if}
