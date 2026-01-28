import { c as create_ssr_component, a as compute_rest_props, b as spread, d as escape_object, e as escape_attribute_value, g as getContext, f as add_attribute, l as createEventDispatcher, v as validate_component, j as each, q as get_store_value, p as onDestroy, h as escape, k as subscribe, o as noop } from "../../../../chunks/ssr.js";
import { u as useDebouncedFn, g as getDeployments, a as getScenarios, c as checkDotrainErrors, F as FileTextarea } from "../../../../chunks/asyncDebounce.js";
import { B as Button, a as codeMirrorTheme } from "../../../../chunks/darkMode.js";
import { b as Table, c as TableHead, a as TableHeadCell, d as TableBody, e as TableBodyRow, T as TableBodyCell, I as Input, R as Refresh, o as orderAddComposeRainlang, f as orderAdd, h as orderAddCalldata, i as ethersExecute, L as Label, M as ModalExecute, v as validateSpecVersion } from "../../../../chunks/order.js";
import { P as PlaySolid, B as BugOutline, T as Tabs, a as TabItem, I as InfoCircleOutline, C as CodeMirrorRainlang } from "../../../../chunks/CodeMirrorRainlang.js";
import { ErrorCode, RawRainlangExtension } from "codemirror-rainlang";
import { invoke } from "@tauri-apps/api";
import { d as derived, w as writable } from "../../../../chunks/index.js";
import { j as cachedWritableStore, r as reportErrorToSentry, k as toasts, l as SentrySeverityLevel, m as Float, b as settingsText, n as globalDotrainFile, A as Alert, B as ButtonLoading } from "../../../../chunks/sentry.js";
import "@observablehq/plot";
import { formatUnits, hexToBigInt } from "viem";
import { sortBy, pickBy, isEmpty, isNil } from "lodash";
import "@tauri-apps/api/clipboard";
import { C as ClipboardOutline, M as Modal, h as walletConnectNetwork, u as useRaindexClient, i as formatEthersTransactionError } from "../../../../chunks/queryClient.js";
import { twMerge } from "tailwind-merge";
import "@square/svelte-store";
import "imask/esm";
import "imask";
import "imask/esm/imask";
import "@sentry/sveltekit";
import "@tauri-apps/api/os";
import "@tauri-apps/api/app";
import "@tauri-apps/api/mocks";
import { createQuery } from "@tanstack/svelte-query";
import Fuse from "fuse.js";
import { p as page } from "../../../../chunks/stores.js";
import * as chains from "viem/chains";
import { p as promiseTimeout } from "../../../../chunks/time.js";
import { P as PageHeader } from "../../../../chunks/PageHeader.js";
import { C as ChevronDownSolid, D as Dropdown, R as Radio } from "../../../../chunks/ChevronDownSolid.js";
import { C as CodeMirror } from "../../../../chunks/CodeMirror.js";
const P = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let colorAndopacity;
  let classP;
  let $$restProps = compute_rest_props($$props, [
    "color",
    "height",
    "align",
    "justify",
    "italic",
    "firstupper",
    "upperClass",
    "opacity",
    "whitespace",
    "size",
    "space",
    "weight"
  ]);
  let { color = "text-gray-900 dark:text-white" } = $$props;
  let { height = "normal" } = $$props;
  let { align = "left" } = $$props;
  let { justify = false } = $$props;
  let { italic = false } = $$props;
  let { firstupper = false } = $$props;
  let { upperClass = "first-line:uppercase first-line:tracking-widest first-letter:text-7xl first-letter:font-bold first-letter:text-gray-900 dark:first-letter:text-gray-100 first-letter:me-3 first-letter:float-left" } = $$props;
  let { opacity = void 0 } = $$props;
  let { whitespace = "normal" } = $$props;
  let { size = "base" } = $$props;
  let { space = void 0 } = $$props;
  let { weight = "normal" } = $$props;
  const sizes = {
    xs: "text-xs",
    sm: "text-sm",
    base: "text-base",
    lg: "text-lg",
    xl: "text-xl",
    "2xl": "text-2xl",
    "3xl": "text-3xl",
    "4xl": "text-4xl",
    "5xl": "text-5xl",
    "6xl": "text-6xl",
    "7xl": "text-7xl",
    "8xl": "text-8xl",
    "9xl": "text-9xl"
  };
  const weights = {
    thin: "font-thin",
    extralight: "font-extralight",
    light: "font-light",
    normal: "font-normal",
    medium: "font-medium",
    semibold: "font-semibold",
    bold: "font-bold",
    extrabold: "font-extrabold",
    black: "font-black"
  };
  const spaces = {
    tighter: "tracking-tighter",
    tight: "tracking-tight",
    normal: "tracking-normal",
    wide: "tracking-wide",
    wider: "tracking-wider",
    widest: "tracking-widest"
  };
  const heights = {
    normal: "leading-normal",
    relaxed: "leading-relaxed",
    loose: "leading-loose"
  };
  const aligns = {
    left: "text-left",
    center: "text-center",
    right: "text-right"
  };
  const whitespaces = {
    normal: "whitespace-normal",
    nowrap: "whitespace-nowrap",
    pre: "whitespace-pre",
    preline: "whitespace-pre-line",
    prewrap: "whitespace-pre-wrap"
  };
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.height === void 0 && $$bindings.height && height !== void 0) $$bindings.height(height);
  if ($$props.align === void 0 && $$bindings.align && align !== void 0) $$bindings.align(align);
  if ($$props.justify === void 0 && $$bindings.justify && justify !== void 0) $$bindings.justify(justify);
  if ($$props.italic === void 0 && $$bindings.italic && italic !== void 0) $$bindings.italic(italic);
  if ($$props.firstupper === void 0 && $$bindings.firstupper && firstupper !== void 0) $$bindings.firstupper(firstupper);
  if ($$props.upperClass === void 0 && $$bindings.upperClass && upperClass !== void 0) $$bindings.upperClass(upperClass);
  if ($$props.opacity === void 0 && $$bindings.opacity && opacity !== void 0) $$bindings.opacity(opacity);
  if ($$props.whitespace === void 0 && $$bindings.whitespace && whitespace !== void 0) $$bindings.whitespace(whitespace);
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.space === void 0 && $$bindings.space && space !== void 0) $$bindings.space(space);
  if ($$props.weight === void 0 && $$bindings.weight && weight !== void 0) $$bindings.weight(weight);
  colorAndopacity = color.split(" ").map((element) => element.trim()).map((element) => element + "/" + String(opacity)).join(" ");
  classP = twMerge(size && sizes[size], opacity && colorAndopacity || color && color, height && heights[height], weight && weights[weight], space && spaces[space], align && aligns[align], justify && "text-justify", italic && "italic", firstupper && upperClass, whitespace && whitespaces[whitespace], $$props.class);
  return `<p${spread([escape_object($$restProps), { class: escape_attribute_value(classP) }], {})}>${slots.default ? slots.default({}) : ``}</p> `;
});
const EditOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "role", "strokeLinecap", "strokeLinejoin", "strokeWidth", "ariaLabel"]);
  const ctx = getContext("iconCtx") ?? {};
  const sizes = {
    xs: "w-3 h-3",
    sm: "w-4 h-4",
    md: "w-5 h-5",
    lg: "w-6 h-6",
    xl: "w-8 h-8"
  };
  let { size = ctx.size || "md" } = $$props;
  let { role = ctx.role || "img" } = $$props;
  let { strokeLinecap = ctx.strokeLinecap || "round" } = $$props;
  let { strokeLinejoin = ctx.strokeLinejoin || "round" } = $$props;
  let { strokeWidth = ctx.strokeWidth || "2" } = $$props;
  let { ariaLabel = "edit outline" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
  if ($$props.strokeLinecap === void 0 && $$bindings.strokeLinecap && strokeLinecap !== void 0) $$bindings.strokeLinecap(strokeLinecap);
  if ($$props.strokeLinejoin === void 0 && $$bindings.strokeLinejoin && strokeLinejoin !== void 0) $$bindings.strokeLinejoin(strokeLinejoin);
  if ($$props.strokeWidth === void 0 && $$bindings.strokeWidth && strokeWidth !== void 0) $$bindings.strokeWidth(strokeWidth);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  return `<svg${spread(
    [
      { xmlns: "http://www.w3.org/2000/svg" },
      { fill: "none" },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge("shrink-0", sizes[size], $$props.class))
      },
      { role: escape_attribute_value(role) },
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      { viewBox: "0 0 20 18" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M15 13v2.833A1.166 1.166 0 0 1 13.833 17H2.167A1.167 1.167 0 0 1 1 15.833V4.167A1.166 1.166 0 0 1 2.167 3h6.616m5.521-.156 2.852 2.852m1.253-4.105a2.017 2.017 0 0 1 0 2.852l-7.844 7.844L7 13l.713-3.565 7.844-7.844a2.016 2.016 0 0 1 2.852 0Z"></path></svg> `;
});
const DropdownRadio = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let optionsSorted;
  const dispatch = createEventDispatcher();
  let { options = {} } = $$props;
  let { value = void 0 } = $$props;
  let open = false;
  if ($$props.options === void 0 && $$bindings.options && options !== void 0) $$bindings.options(options);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    {
      {
        dispatch("change", { value });
        if (value) {
          open = false;
        }
      }
    }
    optionsSorted = sortBy(Object.entries(options), (o) => o[0]);
    $$rendered = `${validate_component(Button, "Button").$$render(
      $$result,
      {
        color: "alternative",
        class: "flex w-full justify-between overflow-hidden overflow-ellipsis pl-2 pr-0 text-left",
        "data-testid": "dropdown-button"
      },
      {},
      {
        default: () => {
          return `<div class="flex-grow overflow-hidden">${slots.content ? slots.content({
            selectedRef: value,
            selectedOption: value !== void 0 ? options[value] : void 0
          }) : ``}</div> ${validate_component(ChevronDownSolid, "ChevronDownSolid").$$render(
            $$result,
            {
              class: "mx-2 h-3 w-3 text-black dark:text-white"
            },
            {},
            {}
          )}`;
        }
      }
    )} ${validate_component(Dropdown, "Dropdown").$$render(
      $$result,
      { "data-testid": "dropdown", open },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `${each(optionsSorted, ([ref, option]) => {
            return `${validate_component(Radio, "Radio").$$render(
              $$result,
              {
                value: ref,
                class: "w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600",
                group: value
              },
              {
                group: ($$value) => {
                  value = $$value;
                  $$settled = false;
                }
              },
              {
                default: () => {
                  return `<div class="ml-2">${slots.option ? slots.option({ option, ref }) : ``}</div> `;
                }
              }
            )}`;
          })}`;
        }
      }
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const css = {
  code: ".ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]{background-color:inherit}",
  map: `{"version":3,"file":"CodeMirrorDotrain.svelte","sources":["CodeMirrorDotrain.svelte"],"sourcesContent":["<script>import CodeMirror from \\"svelte-codemirror-editor\\";\\nimport { RawRainlangExtension } from \\"codemirror-rainlang\\";\\nimport { openLintPanel } from \\"@codemirror/lint\\";\\nexport let rainlangText = void 0;\\nexport let disabled = false;\\nexport let styles = {};\\nexport let rainlangExtension;\\nexport let codeMirrorTheme;\\nexport let onTextChange;\\n<\/script>\\n\\n<div data-testid=\\"codemirror-dotrain\\">\\n\\t<CodeMirror\\n\\t\\tvalue={rainlangText || ''}\\n\\t\\textensions={[rainlangExtension]}\\n\\t\\ttheme={codeMirrorTheme}\\n\\t\\treadonly={disabled}\\n\\t\\tuseTab={true}\\n\\t\\ttabSize={2}\\n\\t\\tstyles={{\\n\\t\\t\\t'&': {\\n\\t\\t\\t\\twidth: '100%'\\n\\t\\t\\t},\\n\\t\\t\\t...styles\\n\\t\\t}}\\n\\t\\ton:change={(e) => {\\n\\t\\t\\tonTextChange(e.detail);\\n\\t\\t}}\\n\\t\\ton:ready={(e) => {\\n\\t\\t\\topenLintPanel(e.detail);\\n\\t\\t}}\\n\\t/>\\n</div>\\n\\n<style global>\\n\\t:global(.ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]) {\\n\\t\\tbackground-color: inherit;\\n\\t}\\n</style>\\n"],"names":[],"mappings":"AAmCS,8CAAgD,CACvD,gBAAgB,CAAE,OACnB"}`
};
const CodeMirrorDotrain = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { rainlangText = void 0 } = $$props;
  let { disabled = false } = $$props;
  let { styles = {} } = $$props;
  let { rainlangExtension } = $$props;
  let { codeMirrorTheme: codeMirrorTheme2 } = $$props;
  let { onTextChange } = $$props;
  if ($$props.rainlangText === void 0 && $$bindings.rainlangText && rainlangText !== void 0) $$bindings.rainlangText(rainlangText);
  if ($$props.disabled === void 0 && $$bindings.disabled && disabled !== void 0) $$bindings.disabled(disabled);
  if ($$props.styles === void 0 && $$bindings.styles && styles !== void 0) $$bindings.styles(styles);
  if ($$props.rainlangExtension === void 0 && $$bindings.rainlangExtension && rainlangExtension !== void 0) $$bindings.rainlangExtension(rainlangExtension);
  if ($$props.codeMirrorTheme === void 0 && $$bindings.codeMirrorTheme && codeMirrorTheme2 !== void 0) $$bindings.codeMirrorTheme(codeMirrorTheme2);
  if ($$props.onTextChange === void 0 && $$bindings.onTextChange && onTextChange !== void 0) $$bindings.onTextChange(onTextChange);
  $$result.css.add(css);
  return `<div data-testid="codemirror-dotrain">${validate_component(CodeMirror, "CodeMirror").$$render(
    $$result,
    {
      value: rainlangText || "",
      extensions: [rainlangExtension],
      theme: codeMirrorTheme2,
      readonly: disabled,
      useTab: true,
      tabSize: 2,
      styles: { "&": { width: "100%" }, ...styles }
    },
    {},
    {}
  )} </div>`;
});
const getBlockNumberFromRpc = async (rpcs) => invoke("get_block_number", { rpcs });
function fetchableStore(key, defaultValue, handleFetch, serialize, deserialize) {
  const value = cachedWritableStore(key, defaultValue, serialize, deserialize);
  const isFetching = writable(false);
  const { subscribe: subscribe2 } = derived([value, isFetching], ([$value, $isFetching]) => ({
    value: $value,
    isFetching: $isFetching
  }));
  async function fetch(data) {
    isFetching.set(true);
    try {
      const res = await handleFetch(data);
      value.set(res);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(e);
    }
    isFetching.set(false);
  }
  return {
    subscribe: subscribe2,
    set: (v) => value.set(v.value),
    fetch
  };
}
const fetchableIntStore = (key, handleFetch) => fetchableStore(
  key,
  0,
  handleFetch,
  (v) => v.toString(),
  (s) => parseInt(s)
);
const forkBlockNumber = fetchableIntStore("forkBlockNumber", async (rpcs) => {
  return getBlockNumberFromRpc(rpcs);
});
async function problemsCallback(rpcs, textDocument, bindings, deployerAddress) {
  try {
    await forkBlockNumber.fetch(rpcs);
    return await invoke("call_lsp_problems", {
      textDocument,
      rpcs,
      blockNumber: get_store_value(forkBlockNumber).value,
      bindings,
      deployer: deployerAddress
    });
  } catch (err) {
    reportErrorToSentry(err, SentrySeverityLevel.Info);
    return [
      {
        msg: typeof err === "string" ? err : err instanceof Error ? err.message : "something went wrong!",
        position: [0, 0],
        code: ErrorCode.NativeParserError
      }
    ];
  }
}
const makeDeploymentsDebugDataMap = async (dotrain, settings, blockNumbers) => invoke("make_deployment_debug", { dotrain, settings, blockNumbers });
const ObservableChart = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { plot } = $$props;
  let { data } = $$props;
  let div;
  if ($$props.plot === void 0 && $$bindings.plot && plot !== void 0) $$bindings.plot(plot);
  if ($$props.data === void 0 && $$bindings.data && data !== void 0) $$bindings.data(data);
  data?.map((row) => Object.fromEntries(Object.entries(row).map(([key, value]) => [key, value.value]))) ?? [];
  return `${`<div role="img" class="w-full border p-4 [&amp;_h2]:text-lg [&amp;_h2]:font-semibold [&amp;_h3]:text-sm" data-testid="chart"${add_attribute("this", div, 0)}></div>`}`;
});
const transformData = (fuzzResult) => {
  if (fuzzResult.data.rows.some((row) => row.length !== fuzzResult.data.columnNames.length)) {
    throw new Error("Number of column names does not match data length");
  }
  return fuzzResult.data.rows.map((row) => {
    const rowObject = {};
    fuzzResult.data.columnNames.forEach((columnName, index) => {
      rowObject[columnName] = [+formatUnits(hexToBigInt(row[index]), 18), row[index]];
    });
    return rowObject;
  });
};
const transformDataForPlot = (fuzzResult) => {
  if (fuzzResult.data.rows.some((row) => row.length !== fuzzResult.data.columnNames.length)) {
    throw new Error("Number of column names does not match data length");
  }
  return fuzzResult.data.rows.map((row) => {
    const rowObject = {};
    fuzzResult.data.columnNames.forEach((columnName, index) => {
      rowObject[columnName] = decodeRainFloat(row[index], columnName);
    });
    return rowObject;
  });
};
const decodeRainFloat = (value, columnName) => {
  const floatResult = Float.tryFromBigint(hexToBigInt(value));
  if (floatResult?.error || !floatResult?.value) {
    const message = floatResult?.error?.readableMsg ?? floatResult?.error?.msg ?? "Unknown error";
    throw new Error(`Failed to parse ${columnName} value: ${message}`);
  }
  const formattedResult = floatResult.value.formatWithScientific(false);
  if (formattedResult.error || !formattedResult.value) {
    const message = formattedResult.error?.readableMsg ?? formattedResult.error?.msg ?? "Unknown error";
    throw new Error(`Failed to format ${columnName} value: ${message}`);
  }
  const numericValue = Number(formattedResult.value);
  if (!Number.isFinite(numericValue)) {
    throw new Error(`Value for ${columnName} is not a finite number: ${formattedResult.value}`);
  }
  return {
    float: floatResult.value,
    formatted: formattedResult.value,
    value: numericValue
  };
};
const MetricChart = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let metricDatum;
  let metricFormattedValue;
  let metricValueWithUnits;
  let { metric } = $$props;
  let { data } = $$props;
  onDestroy(() => {
  });
  if ($$props.metric === void 0 && $$bindings.metric && metric !== void 0) $$bindings.metric(metric);
  if ($$props.data === void 0 && $$bindings.data && data !== void 0) $$bindings.data(data);
  metricDatum = data?.[0]?.[metric.value];
  metricFormattedValue = metricDatum?.formatted ?? "";
  metricValueWithUnits = metricFormattedValue === "" ? "" : `${metric?.["unit-prefix"] ?? ""}${metricFormattedValue}${metric?.["unit-suffix"] ?? ""}`;
  return `<div class="flex h-full w-full flex-col items-center justify-between border p-4"><span>${escape(metric.label)}</span> <div class="relative w-full"><button type="button" class="relative flex w-full items-center justify-center gap-2 rounded bg-transparent px-2 py-1 text-2xl transition hover:bg-slate-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-500 dark:hover:bg-slate-800"${add_attribute("aria-label", `Copy ${metric.label} value`, 0)}${add_attribute("title", metricValueWithUnits || "", 0)}><span class="block w-full truncate pr-6 text-center">${escape(metricValueWithUnits ?? "")}</span> ${validate_component(ClipboardOutline, "ClipboardOutline").$$render(
    $$result,
    {
      class: "absolute right-2 top-1/2 h-5 w-5 -translate-y-1/2 text-slate-500 dark:text-slate-300",
      "aria-hidden": "true"
    },
    {},
    {}
  )}</button> ${``}</div> ${metric?.description ? `<span class="text-sm">${escape(metric.description)}</span>` : ``}</div>`;
});
const Charts = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { chartData } = $$props;
  if ($$props.chartData === void 0 && $$bindings.chartData && chartData !== void 0) $$bindings.chartData(chartData);
  return `${chartData ? `<div class="mt-8 flex flex-col items-center gap-y-6">${each(sortBy(Object.entries(chartData.charts), ["0"]), (chart) => {
    let data = transformDataForPlot(chartData.scenariosData[chart[1].scenario.key]);
    return ` <div class="w-full"><div class="flex flex-col justify-center gap-y-4"><h2 class="text-2xl font-bold">${escape(chart[0])}</h2> <div class="grid w-full grid-cols-2 gap-4">${each(chart[1]?.metrics || [], (metric) => {
      return `<div class="col-span-1 flex flex-col gap-y-4">${validate_component(MetricChart, "MetricChart").$$render($$result, { metric, data }, {}, {})} </div>`;
    })} ${each(chart[1]?.plots || [], (plot) => {
      return `<div class="col-span-1 flex flex-col gap-y-4">${validate_component(ObservableChart, "ObservableChart").$$render($$result, { plot, data }, {}, {})} </div>`;
    })} </div></div> </div>`;
  })}</div>` : `No scenario data`}`;
});
function pickScenarios(scenarios, chainId) {
  return pickBy(scenarios, (d) => d.deployer.network.chainId === chainId);
}
const ModalDebugContext = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { open = false } = $$props;
  let { networks = void 0 } = $$props;
  let { blockNumbers = {} } = $$props;
  let { onClose } = $$props;
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.networks === void 0 && $$bindings.networks && networks !== void 0) $$bindings.networks(networks);
  if ($$props.blockNumbers === void 0 && $$bindings.blockNumbers && blockNumbers !== void 0) $$bindings.blockNumbers(blockNumbers);
  if ($$props.onClose === void 0 && $$bindings.onClose && onClose !== void 0) $$bindings.onClose(onClose);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: "Debug Block Height",
        outsideclose: true,
        size: "sm",
        backdropClass: "fixed inset-0 z-40 bg-gray-900 bg-opacity-50 dark:bg-opacity-80 z-[1000] backdrop-class-id",
        dialogClass: "fixed top-0 start-0 end-0 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex z-[1000]",
        open
      },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `${networks ? `${validate_component(Table, "Table").$$render(
            $$result,
            {
              divClass: "rounded-lg overflow-hidden dark:border-none border overflow-x-scroll"
            },
            {},
            {
              default: () => {
                return `${validate_component(TableHead, "TableHead").$$render($$result, {}, {}, {
                  default: () => {
                    return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                      default: () => {
                        return `Network`;
                      }
                    })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                      default: () => {
                        return `Block Height`;
                      }
                    })}`;
                  }
                })} ${validate_component(TableBody, "TableBody").$$render($$result, {}, {}, {
                  default: () => {
                    return `${each(Object.entries(networks ?? {}).sort((a, b) => Number(a[0]) > Number(b[0]) ? 1 : -1), ([chainId, networkName]) => {
                      return `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                        default: () => {
                          return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { "data-testid": `network-name-${chainId}` }, {}, {
                            default: () => {
                              return `${escape(networkName)}`;
                            }
                          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                            default: () => {
                              return `${validate_component(Input, "Input").$$render(
                                $$result,
                                {
                                  type: "number",
                                  size: "sm",
                                  class: "self-center",
                                  placeholder: "Enter Block Height",
                                  value: blockNumbers[Number(chainId)],
                                  "data-testid": `chain-block-${chainId}`
                                },
                                {},
                                {}
                              )} `;
                            }
                          })} `;
                        }
                      })}`;
                    })}`;
                  }
                })}`;
              }
            }
          )}` : `Found no deployment, please add deployments to your order&#39;s configurations to debug it`}`;
        }
      }
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const ScenarioDebugTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let queryKey;
  let scenarioDebugQuery;
  let $scenarioDebugQuery, $$unsubscribe_scenarioDebugQuery = noop, $$subscribe_scenarioDebugQuery = () => ($$unsubscribe_scenarioDebugQuery(), $$unsubscribe_scenarioDebugQuery = subscribe(scenarioDebugQuery, ($$value) => $scenarioDebugQuery = $$value), scenarioDebugQuery);
  let $settingsText, $$unsubscribe_settingsText;
  let $globalDotrainFile, $$unsubscribe_globalDotrainFile;
  let $queryKey, $$unsubscribe_queryKey = noop, $$subscribe_queryKey = () => ($$unsubscribe_queryKey(), $$unsubscribe_queryKey = subscribe(queryKey, ($$value) => $queryKey = $$value), queryKey);
  $$unsubscribe_settingsText = subscribe(settingsText, (value) => $settingsText = value);
  $$unsubscribe_globalDotrainFile = subscribe(globalDotrainFile, (value) => $globalDotrainFile = value);
  let openDebugBlockNumberModal = false;
  let blockNumbers = {};
  let { networks } = $$props;
  let displayData = void 0;
  const fetchData = async () => {
    const res = await makeDeploymentsDebugDataMap($queryKey[0], $settingsText, blockNumbers);
    for (const deploymentKey in res.dataMap) {
      blockNumbers[res.dataMap[deploymentKey].chainId] = res.dataMap[deploymentKey].blockNumber;
    }
    return res;
  };
  const fileUpdate = async (dotrain, settings) => {
    queryKey.set([dotrain, settings]);
  };
  const { debouncedFn: debounceFileUpdate } = useDebouncedFn(fileUpdate, 500);
  const handleRefresh = () => {
    $scenarioDebugQuery.refetch();
  };
  if ($$props.networks === void 0 && $$bindings.networks && networks !== void 0) $$bindings.networks(networks);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$subscribe_queryKey(queryKey = writable([$globalDotrainFile.text, $settingsText]));
    {
      debounceFileUpdate($globalDotrainFile.text, $settingsText);
    }
    $$subscribe_scenarioDebugQuery(scenarioDebugQuery = createQuery({
      queryKey: $queryKey,
      queryFn: fetchData,
      refetchOnWindowFocus: false,
      enabled: $globalDotrainFile.text !== "" && $settingsText !== "",
      refetchInterval: false
    }));
    {
      {
        if (!$scenarioDebugQuery.isError && $scenarioDebugQuery.data) {
          displayData = $scenarioDebugQuery.data.dataMap;
        } else if ($globalDotrainFile.text === "" || $settingsText === "") {
          displayData = void 0;
        }
      }
    }
    $$rendered = `<div class="flex items-center justify-end"><div class="flex items-center gap-x-1">${$scenarioDebugQuery.isError ? `<div class="text-red-500">${escape($scenarioDebugQuery.error)}</div>` : ``} <span></span> <button type="button" class="mr-2 flex items-center text-sm text-gray-500 hover:text-gray-700 focus:outline-none"><span class="mr-1" data-svelte-h="svelte-1fyasmx">Change block heights</span> ${validate_component(EditOutline, "EditOutline").$$render($$result, {}, {}, {})}</button> ${validate_component(Refresh, "Refresh").$$render(
      $$result,
      {
        "data-testid": "refreshButton",
        class: "h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400",
        spin: $scenarioDebugQuery.isFetching
      },
      {},
      {}
    )} <button class="ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400">${`${validate_component(PlaySolid, "PlaySolid").$$render($$result, {}, {}, {})}`}</button></div></div> ${!$scenarioDebugQuery.error && displayData ? `${each(Object.entries(displayData).sort((a, b) => a[1].chainId > b[1].chainId ? 1 : -1), ([deploymentName, results]) => {
      return `<h2 class="text-md my-4">Deployment: <strong>${escape(deploymentName)}</strong></h2> ${validate_component(Table, "Table").$$render(
        $$result,
        {
          divClass: "rounded-lg overflow-hidden dark:border-none border overflow-x-scroll"
        },
        {},
        {
          default: () => {
            return `${validate_component(TableHead, "TableHead").$$render($$result, {}, {}, {
              default: () => {
                return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Order`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Scenario`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Pair`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Maximum Output`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Ratio`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                  default: () => {
                    return `Block Height`;
                  }
                })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { class: "w-[50px]" }, {}, {})} `;
              }
            })} ${validate_component(TableBody, "TableBody").$$render($$result, {}, {}, {
              default: () => {
                return `${each(results.pairsData, (item) => {
                  return `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                    default: () => {
                      return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                        default: () => {
                          return `${escape(item.order)}`;
                        }
                      })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                        default: () => {
                          return `${escape(item.scenario)}`;
                        }
                      })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                        default: () => {
                          return `${escape(item.pair)}`;
                        }
                      })} ${item.result ? (() => {
                        let fuzzResult = item.result, data = transformData(fuzzResult)[0], dataEntries = Object.entries(data), keyRegex = /^\d+\.\d+$/, mainEntries = dataEntries.filter(([key]) => keyRegex.test(key));
                        return `     ${mainEntries.length < 2 ? `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { colspan: "2", class: "text-red-500" }, {}, {
                          default: () => {
                            return `Missing stack data for max output and ratio`;
                          }
                        })}` : (() => {
                          let maxOutput = mainEntries[mainEntries.length - 2], ioRatio = mainEntries[mainEntries.length - 1], denominator = BigInt(ioRatio?.[1]?.[1] ?? 0n);
                          return `   ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                            default: () => {
                              return `${escape(maxOutput[1][0])} `;
                            }
                          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                            default: () => {
                              return `${escape(ioRatio[1][0])} <span class="text-gray-400">(${escape(denominator === 0n ? "0" : formatUnits(10n ** 36n / denominator, 18))})</span> `;
                            }
                          })}`;
                        })()} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                          default: () => {
                            return `${escape(results.blockNumber)}`;
                          }
                        })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                          default: () => {
                            return `<button>${validate_component(BugOutline, "BugOutline").$$render($$result, { size: "sm", color: "grey" }, {}, {})}</button> `;
                          }
                        })}`;
                      })() : `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { colspan: "5", class: "text-red-500" }, {}, {
                        default: () => {
                          return `${escape(item.error)}`;
                        }
                      })}`} `;
                    }
                  })}`;
                })} `;
              }
            })} `;
          }
        }
      )}`;
    })}` : ``} ${validate_component(ModalDebugContext, "ModalDebugContext").$$render(
      $$result,
      {
        onClose: handleRefresh,
        open: openDebugBlockNumberModal,
        blockNumbers,
        networks
      },
      {
        open: ($$value) => {
          openDebugBlockNumberModal = $$value;
          $$settled = false;
        },
        blockNumbers: ($$value) => {
          blockNumbers = $$value;
          $$settled = false;
        },
        networks: ($$value) => {
          networks = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  $$unsubscribe_scenarioDebugQuery();
  $$unsubscribe_settingsText();
  $$unsubscribe_globalDotrainFile();
  $$unsubscribe_queryKey();
  return $$rendered;
});
const WordTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let filteredWords;
  let { authoringMeta } = $$props;
  let { pragma } = $$props;
  let search;
  const fuse = new Fuse(
    authoringMeta.words,
    {
      keys: ["word", "description"],
      ignoreLocation: true,
      threshold: 0
    }
  );
  if ($$props.authoringMeta === void 0 && $$bindings.authoringMeta && authoringMeta !== void 0) $$bindings.authoringMeta(authoringMeta);
  if ($$props.pragma === void 0 && $$bindings.pragma && pragma !== void 0) $$bindings.pragma(pragma);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    filteredWords = search ? fuse.search(search).map((res) => res.item) : authoringMeta.words;
    $$rendered = `${validate_component(Table, "Table").$$render(
      $$result,
      {
        divClass: "cursor-pointer rounded-lg dark:border-none border h-[500px] overflow-y-scroll relative w-[450px] bg-white dark:bg-gray-800",
        "data-testid": `word-table-${pragma}`
      },
      {},
      {
        default: () => {
          return `${validate_component(TableHead, "TableHead").$$render($$result, { theadClass: "sticky top-0" }, {}, {
            default: () => {
              return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
                default: () => {
                  return `<div class="flex flex-col text-xs font-normal"><div data-testid="pragma" class="mb-3 mt-1">From ${escape(pragma)}</div> ${validate_component(Input, "Input").$$render(
                    $$result,
                    {
                      "data-testid": "search-input",
                      placeholder: "Search words",
                      value: search
                    },
                    {
                      value: ($$value) => {
                        search = $$value;
                        $$settled = false;
                      }
                    },
                    {}
                  )}</div>`;
                }
              })}`;
            }
          })} ${validate_component(TableBody, "TableBody").$$render($$result, { tableBodyClass: "w-full" }, {}, {
            default: () => {
              return `${filteredWords.length === 0 ? `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                default: () => {
                  return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `<div data-testid="no-results-msg" class="text-center text-gray-500" data-svelte-h="svelte-1sdgh4g">No words found</div>`;
                    }
                  })}`;
                }
              })}` : `${each(filteredWords, (word) => {
                return `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                  default: () => {
                    return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                      default: () => {
                        return `<div class="flex flex-col gap-y-2"><div data-testid="word">${escape(word.word)}</div> <div data-testid="description" class="whitespace-normal text-gray-500">${escape(word.description)} </div></div> `;
                      }
                    })} `;
                  }
                })}`;
              })}`}`;
            }
          })}`;
        }
      }
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const Words = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { authoringMetas } = $$props;
  let { error } = $$props;
  if ($$props.authoringMetas === void 0 && $$bindings.authoringMetas && authoringMetas !== void 0) $$bindings.authoringMetas(authoringMetas);
  if ($$props.error === void 0 && $$bindings.error && error !== void 0) $$bindings.error(error);
  return `${authoringMetas ? `${validate_component(Tabs, "Tabs").$$render(
    $$result,
    {
      style: "underline",
      contentClass: "mt-4",
      defaultClass: "flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
    },
    {},
    {
      default: () => {
        return `${each(authoringMetas, (scenario) => {
          return `${validate_component(TabItem, "TabItem").$$render($$result, { title: scenario.scenario }, {}, {
            default: () => {
              return `<div class="flex gap-x-2 text-sm">${scenario.deployerWords.words.type === "Success" ? `${validate_component(WordTable, "WordTable").$$render(
                $$result,
                {
                  authoringMeta: scenario.deployerWords.words.data,
                  pragma: scenario.deployerWords.address
                },
                {},
                {}
              )}` : `${scenario.deployerWords.words.type === "Error" ? `<div data-testid="deployer-error-msg" class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800"><p data-svelte-h="svelte-5exj2y">Error getting words for this deployer:</p> <p>${escape(scenario.deployerWords.words.data)}</p> </div>` : ``}`} ${each(scenario.pragmaWords, (pragma) => {
                return `${pragma.words.type === "Success" ? `${validate_component(WordTable, "WordTable").$$render(
                  $$result,
                  {
                    authoringMeta: pragma.words.data,
                    pragma: pragma.address
                  },
                  {},
                  {}
                )}` : `${pragma.words.type === "Error" ? `<div class="relative h-[500px] w-[450px] overflow-y-scroll rounded-lg border bg-white p-4 dark:border-none dark:bg-gray-800" data-testid="pragma-error-msg"><p>Error getting words for the pragma ${escape(pragma.address)}:</p> <p>${escape(pragma.words.data)}</p> </div>` : ``}`}`;
              })}</div> `;
            }
          })}`;
        })}`;
      }
    }
  )}` : `${error ? `<div data-testid="error-msg">${validate_component(P, "P").$$render($$result, {}, {}, {
    default: () => {
      return `Error getting words for this order`;
    }
  })} ${validate_component(P, "P").$$render($$result, {}, {}, {
    default: () => {
      return `${escape(error?.toString() || "")}`;
    }
  })}</div>` : ``}`}`;
});
const getAuthoringMetaV2ForScenarios = async (dotrain, settings) => invoke("get_authoring_meta_v2_for_scenarios", { dotrain, settings });
const SpecVersionValidator = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { error } = $$props;
  if ($$props.error === void 0 && $$bindings.error && error !== void 0) $$bindings.error(error);
  return `${error && typeof error == "string" && error.startsWith("Spec version") ? `${validate_component(Alert, "Alert").$$render($$result, { color: "red" }, {}, {
    icon: () => {
      return `${validate_component(InfoCircleOutline, "InfoCircleOutline").$$render($$result, { slot: "icon", class: "h-5 w-5" }, {}, {})}`;
    },
    default: () => {
      return `<div class="flex flex-col"><span>${escape(error)}</span> <span data-svelte-h="svelte-ltt4n8">This order may not be compatible with this version of Raindex. Head to
        <a class="underline" href="https://github.com/rainlanguage/rain.orderbook/releases">Github</a> to find a compatible release.</span></div>`;
    }
  })}` : ``}`;
});
async function executeWalletConnectOrder(dotrainText, deployment, dependencies) {
  const {
    orderAddCalldataFn,
    ethersExecuteFn,
    reportErrorToSentryFn,
    formatEthersTransactionErrorFn,
    successToastFn,
    errorToastFn
  } = dependencies;
  if (isEmpty(deployment.order.orderbook)) throw Error("No orderbook associated with scenario");
  try {
    const calldata = await orderAddCalldataFn(dotrainText, deployment);
    const tx = await ethersExecuteFn(calldata, deployment.order.orderbook.address);
    await tx.wait(1);
    successToastFn("Transaction sent successfully!");
  } catch (e) {
    reportErrorToSentryFn(e);
    errorToastFn(formatEthersTransactionErrorFn(e));
    throw e;
  }
}
async function executeLedgerOrder(dotrainText, deployment, orderAddFn, reportErrorToSentryFn) {
  if (isEmpty(deployment.order?.orderbook)) throw Error("No orderbook associated with scenario");
  try {
    await orderAddFn(dotrainText, deployment);
  } catch (e) {
    reportErrorToSentryFn(e);
    throw e;
  }
}
async function generateRainlangStrings(dotrainText, settingsStrings, scenarios) {
  try {
    if (isEmpty(scenarios)) return void 0;
    const composedRainlangForScenarios = /* @__PURE__ */ new Map();
    for (const scenario of Object.values(scenarios)) {
      try {
        const composedRainlang = await orderAddComposeRainlang(
          dotrainText,
          settingsStrings,
          scenario
        );
        composedRainlangForScenarios.set(scenario, composedRainlang);
      } catch (e) {
        composedRainlangForScenarios.set(
          scenario,
          e?.toString() || `Error composing rainlang for scenario: ${scenario.key}`
        );
      }
    }
    return composedRainlangForScenarios;
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Error);
    return void 0;
  }
}
function getDeploymentsNetworks(deployments) {
  if (deployments) {
    const networks = {};
    for (const key in deployments) {
      const chainId = deployments[key].scenario.deployer.network.chainId;
      if (!networks[chainId]) {
        const networkKey = Object.values(chains).find((v) => v.id === chainId)?.name ?? deployments[key].scenario.deployer.network.key;
        networks[chainId] = networkKey;
      }
    }
    if (!Object.keys(networks).length) return void 0;
    else return networks;
  }
  return void 0;
}
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let deployment;
  let scenarios;
  let bindings;
  let rainlangExtension;
  let deploymentNetworks;
  let $settingsText, $$unsubscribe_settingsText;
  let $globalDotrainFile, $$unsubscribe_globalDotrainFile;
  let $walletConnectNetwork, $$unsubscribe_walletConnectNetwork;
  let $page, $$unsubscribe_page;
  let $specVersionError, $$unsubscribe_specVersionError;
  let $codeMirrorTheme, $$unsubscribe_codeMirrorTheme;
  let $generatedRainlang, $$unsubscribe_generatedRainlang;
  let $error, $$unsubscribe_error;
  let $authoringMetasResult, $$unsubscribe_authoringMetasResult;
  let $authoringMetasError, $$unsubscribe_authoringMetasError;
  $$unsubscribe_settingsText = subscribe(settingsText, (value) => $settingsText = value);
  $$unsubscribe_globalDotrainFile = subscribe(globalDotrainFile, (value) => $globalDotrainFile = value);
  $$unsubscribe_walletConnectNetwork = subscribe(walletConnectNetwork, (value) => $walletConnectNetwork = value);
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  $$unsubscribe_codeMirrorTheme = subscribe(codeMirrorTheme, (value) => $codeMirrorTheme = value);
  const raindexClient = useRaindexClient();
  let isSubmitting = false;
  let isCharting = false;
  let chartData;
  let deploymentRef = void 0;
  let scenarioRef = void 0;
  let openAddOrderModal = false;
  let allDeployments = {};
  let allScenarios = {};
  let openTab = {};
  const { debouncedFn: debounceGetAuthoringMetas, result: authoringMetasResult, error: authoringMetasError } = useDebouncedFn(getAuthoringMetaV2ForScenarios, 500);
  $$unsubscribe_authoringMetasResult = subscribe(authoringMetasResult, (value) => $authoringMetasResult = value);
  $$unsubscribe_authoringMetasError = subscribe(authoringMetasError, (value) => $authoringMetasError = value);
  const { debouncedFn: debouncedGenerateRainlangStrings, result: generatedRainlang, error } = useDebouncedFn(generateRainlangStrings, 500);
  $$unsubscribe_generatedRainlang = subscribe(generatedRainlang, (value) => $generatedRainlang = value);
  $$unsubscribe_error = subscribe(error, (value) => $error = value);
  async function handleExecuteLedger() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error("Select a deployment to add order");
      await executeLedgerOrder($globalDotrainFile.text, deployment, orderAdd, reportErrorToSentry);
    } catch (e) {
      toasts.error(e.message || "Ledger execution failed");
    }
    isSubmitting = false;
  }
  async function handleExecuteWalletConnect() {
    isSubmitting = true;
    try {
      if (!deployment) throw Error("Select a deployment to add order");
      await executeWalletConnectOrder($globalDotrainFile.text, deployment, {
        orderAddCalldataFn: async (dotrain, deploy) => await orderAddCalldata(dotrain, deploy),
        ethersExecuteFn: ethersExecute,
        reportErrorToSentryFn: reportErrorToSentry,
        formatEthersTransactionErrorFn: formatEthersTransactionError,
        successToastFn: toasts.success,
        errorToastFn: toasts.error
      });
    } catch {
    }
    isSubmitting = false;
  }
  const { debouncedFn: debounceValidateSpecVersion, error: specVersionError } = useDebouncedFn(validateSpecVersion, 500);
  $$unsubscribe_specVersionError = subscribe(specVersionError, (value) => $specVersionError = value);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    {
      if ($globalDotrainFile) {
        getDeployments().then((deployments) => allDeployments = deployments);
        getScenarios().then((scenarios2) => allScenarios = scenarios2);
      }
    }
    {
      if (deploymentRef && allDeployments && !Object.keys(allDeployments).includes(deploymentRef)) {
        deploymentRef = void 0;
      }
    }
    deployment = deploymentRef ? allDeployments[deploymentRef] : void 0;
    scenarios = pickScenarios(allScenarios, $walletConnectNetwork);
    bindings = deployment ? deployment.scenario.bindings : {};
    {
      debounceGetAuthoringMetas($globalDotrainFile.text, [$settingsText]);
    }
    {
      debouncedGenerateRainlangStrings($globalDotrainFile.text, [$settingsText], allScenarios);
    }
    rainlangExtension = new RawRainlangExtension({
      diagnostics: async (text) => {
        let configProblems = [];
        let problems = [];
        try {
          configProblems = await checkDotrainErrors(text.text, [$settingsText]);
        } catch (e) {
          configProblems = [{ msg: e, position: [0, 0], code: 9 }];
        }
        try {
          const network = raindexClient.getNetworkByChainId($walletConnectNetwork);
          if (network.error) {
            throw new Error(network.error.readableMsg);
          }
          problems = await promiseTimeout(problemsCallback(network.value.rpcs, text, bindings, deployment?.scenario.deployer.address), 5e3, "failed to parse on native parser");
        } catch (e) {
          problems = [{ msg: e, position: [0, 0], code: 9 }];
        }
        return [...configProblems, ...problems];
      }
    });
    {
      {
        if (isNil(scenarioRef) && !isEmpty(scenarios)) {
          scenarioRef = Object.keys(scenarios)[0];
        }
      }
    }
    {
      debounceValidateSpecVersion($globalDotrainFile.text, [$settingsText]);
    }
    deploymentNetworks = getDeploymentsNetworks(allDeployments);
    $$rendered = `${validate_component(PageHeader, "PageHeader").$$render(
      $$result,
      {
        title: "Add Order",
        pathname: $page.url.pathname
      },
      {},
      {}
    )} ${validate_component(FileTextarea, "FileTextarea").$$render($$result, { textFile: globalDotrainFile }, {}, {
      additionalFields: () => {
        return `<div class="flex items-center justify-end gap-x-4">${isEmpty(allDeployments) ? `<span class="text-gray-500 dark:text-gray-400" data-svelte-h="svelte-1id0c6b">No valid deployments found</span>` : `${validate_component(Label, "Label").$$render($$result, { class: "whitespace-nowrap" }, {}, {
          default: () => {
            return `Select deployment`;
          }
        })} ${validate_component(DropdownRadio, "DropdownRadio").$$render(
          $$result,
          {
            options: allDeployments,
            value: deploymentRef
          },
          {
            value: ($$value) => {
              deploymentRef = $$value;
              $$settled = false;
            }
          },
          {
            option: ({ ref }) => {
              return `<div class="w-full overflow-hidden overflow-ellipsis"><div class="text-md break-word mb-2">${escape(ref)}</div></div> `;
            },
            content: ({ selectedRef }) => {
              return `<span>${escape(!isNil(selectedRef) ? selectedRef : "Select a deployment")}</span>`;
            }
          }
        )}`} ${validate_component(ButtonLoading, "ButtonLoading").$$render(
          $$result,
          {
            class: "min-w-fit",
            color: "green",
            loading: isSubmitting,
            disabled: $globalDotrainFile.isEmpty || isNil(deploymentRef) || !!$specVersionError
          },
          {},
          {
            default: () => {
              return `Add Order`;
            }
          }
        )}</div> `;
      },
      textarea: () => {
        return `${validate_component(CodeMirrorDotrain, "CodeMirrorDotrain").$$render(
          $$result,
          {
            codeMirrorTheme: $codeMirrorTheme,
            rainlangText: $globalDotrainFile.text,
            disabled: isSubmitting,
            styles: { "&": { minHeight: "400px" } },
            rainlangExtension,
            onTextChange: (text) => globalDotrainFile.set({ ...$globalDotrainFile, text })
          },
          {},
          {}
        )} `;
      },
      alert: () => {
        return `${validate_component(SpecVersionValidator, "SpecVersionValidator").$$render($$result, { error: $specVersionError }, {}, {})}`;
      }
    })} ${validate_component(Button, "Button").$$render(
      $$result,
      {
        disabled: isCharting,
        size: "sm",
        class: "self-center"
      },
      {},
      {
        default: () => {
          return `<span class="mr-2" data-svelte-h="svelte-2lgb5">Generate charts</span>${``}`;
        }
      }
    )} ${validate_component(Tabs, "Tabs").$$render(
      $$result,
      {
        style: "underline",
        contentClass: "mt-4",
        defaultClass: "flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
      },
      {},
      {
        default: () => {
          return `${validate_component(TabItem, "TabItem").$$render($$result, { open: true, title: "Rainlang" }, {}, {
            default: () => {
              return `${$generatedRainlang && !$error ? `${validate_component(Tabs, "Tabs").$$render(
                $$result,
                {
                  style: "underline",
                  contentClass: "mt-4",
                  defaultClass: "flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
                },
                {},
                {
                  default: () => {
                    return `${each(Array.from($generatedRainlang.entries()), ([scenario, rainlangText]) => {
                      return `${validate_component(TabItem, "TabItem").$$render(
                        $$result,
                        {
                          title: scenario.key,
                          open: openTab[scenario.key]
                        },
                        {
                          open: ($$value) => {
                            openTab[scenario.key] = $$value;
                            $$settled = false;
                          }
                        },
                        {
                          default: () => {
                            return `${validate_component(CodeMirrorRainlang, "CodeMirrorRainlang").$$render(
                              $$result,
                              {
                                rainlangText,
                                codeMirrorDisabled: true,
                                codeMirrorTheme: $codeMirrorTheme
                              },
                              {},
                              {}
                            )} `;
                          }
                        }
                      )}`;
                    })}`;
                  }
                }
              )}` : ``}`;
            }
          })} ${validate_component(TabItem, "TabItem").$$render($$result, { title: "Debug" }, {}, {
            default: () => {
              return `${validate_component(ScenarioDebugTable, "ScenarioDebugTable").$$render(
                $$result,
                { networks: deploymentNetworks },
                {
                  networks: ($$value) => {
                    deploymentNetworks = $$value;
                    $$settled = false;
                  }
                },
                {}
              )}`;
            }
          })} ${validate_component(TabItem, "TabItem").$$render($$result, { title: "Charts" }, {}, {
            default: () => {
              return `${validate_component(Charts, "Charts").$$render($$result, { chartData }, {}, {})}`;
            }
          })} ${validate_component(TabItem, "TabItem").$$render($$result, { title: "Words" }, {}, {
            default: () => {
              return `${validate_component(Words, "Words").$$render(
                $$result,
                {
                  authoringMetas: $authoringMetasResult,
                  error: $authoringMetasError
                },
                {},
                {}
              )}`;
            }
          })}`;
        }
      }
    )} ${validate_component(ModalExecute, "ModalExecute").$$render(
      $$result,
      {
        chainId: deployment?.order.network.chainId,
        title: "Add Order",
        execButtonLabel: "Add Order",
        executeWalletconnect: handleExecuteWalletConnect,
        executeLedger: handleExecuteLedger,
        open: openAddOrderModal,
        isSubmitting
      },
      {
        open: ($$value) => {
          openAddOrderModal = $$value;
          $$settled = false;
        },
        isSubmitting: ($$value) => {
          isSubmitting = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  $$unsubscribe_settingsText();
  $$unsubscribe_globalDotrainFile();
  $$unsubscribe_walletConnectNetwork();
  $$unsubscribe_page();
  $$unsubscribe_specVersionError();
  $$unsubscribe_codeMirrorTheme();
  $$unsubscribe_generatedRainlang();
  $$unsubscribe_error();
  $$unsubscribe_authoringMetasResult();
  $$unsubscribe_authoringMetasError();
  return $$rendered;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
