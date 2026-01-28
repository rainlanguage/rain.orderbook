import { c as create_ssr_component, a as compute_rest_props, g as getContext, b as spread, d as escape_object, e as escape_attribute_value, v as validate_component, h as escape, f as add_attribute, k as subscribe, p as onDestroy, l as createEventDispatcher, j as each, o as noop, n as getAllContexts } from "../../../../chunks/ssr.js";
import { p as page } from "../../../../chunks/stores.js";
import { B as Button, c as colorTheme, a as codeMirrorTheme, l as lightweightChartsTheme } from "../../../../chunks/darkMode.js";
import { B as ButtonGroup, c as QKEY_ORDER_TRADES_LIST, D as DEFAULT_PAGE_SIZE, T as TanstackAppTable, d as QKEY_ORDER_QUOTE, u as useAccount, e as QKEY_ORDER, f as handleQuoteDebugModal, g as handleDebugTradeModal, h as handleOrderRemoveModal, i as handleDepositModal, j as handleWithdrawModal } from "../../../../chunks/modal.js";
import { createQuery, createInfiniteQuery, useQueryClient } from "@tanstack/svelte-query";
import { T as Tooltip_1, H as Hash, e as HashType, C as ClipboardOutline, u as useRaindexClient, W as WalletOutline, g as Tooltip } from "../../../../chunks/queryClient.js";
import { i as invalidateTanstackQueries, T as TanstackPageContentDetail, C as CardProperty, g as getExplorerLink, A as ArrowDownToBracketOutline } from "../../../../chunks/getExplorerLink.js";
import { P as PageHeader } from "../../../../chunks/PageHeader.js";
import { B as Badge } from "../../../../chunks/Badge.js";
import { S as Spinner } from "../../../../chunks/sentry.js";
import { t as timestampSecondsToUTCTimestamp, f as formatTimestampSecondsAsLocal } from "../../../../chunks/time.js";
import { sortBy } from "lodash";
import "lightweight-charts";
import { T as TableBodyCell, a as TableHeadCell, R as Refresh, b as Table, c as TableHead, d as TableBody, e as TableBodyRow } from "../../../../chunks/order.js";
import { B as BugOutline, P as PlaySolid, T as Tabs, a as TabItem, C as CodeMirrorRainlang, I as InfoCircleOutline } from "../../../../chunks/CodeMirrorRainlang.js";
import { u as useToasts } from "../../../../chunks/useToasts.js";
import { twMerge } from "tailwind-merge";
import { toHex, hexToNumber, isHex } from "viem";
import { A as ArrowUpFromBracketOutline } from "../../../../chunks/ArrowUpFromBracketOutline.js";
const PauseSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "role", "ariaLabel"]);
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
  let { ariaLabel = "pause solid" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  return `<svg${spread(
    [
      { xmlns: "http://www.w3.org/2000/svg" },
      { fill: "currentColor" },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge("shrink-0", sizes[size], $$props.class))
      },
      { role: escape_attribute_value(role) },
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      { viewBox: "0 0 12 16" }
    ],
    {}
  )}><path fill="currentColor" d="M3 0H2a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2Zm7 0H9a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2Z"></path></svg> `;
});
const PenSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "role", "ariaLabel"]);
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
  let { ariaLabel = "pen solid" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  return `<svg${spread(
    [
      { xmlns: "http://www.w3.org/2000/svg" },
      { fill: "currentColor" },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge("shrink-0", sizes[size], $$props.class))
      },
      { role: escape_attribute_value(role) },
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      { viewBox: "0 0 20 20" }
    ],
    {}
  )}><path fill="currentColor" d="m13.835 7.578-.005.007-7.137 7.137 2.139 2.138 7.143-7.142-2.14-2.14Zm-10.696 3.59 2.139 2.14 7.138-7.137.007-.005-2.141-2.141-7.143 7.143Zm1.433 4.261L2 12.852.051 18.684a1 1 0 0 0 1.265 1.264L7.147 18l-2.575-2.571Zm14.249-14.25a4.03 4.03 0 0 0-5.693 0L11.7 2.611 17.389 8.3l1.432-1.432a4.029 4.029 0 0 0 0-5.689Z"></path></svg> `;
});
const BadgeActive = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { active } = $$props;
  if ($$props.active === void 0 && $$bindings.active && active !== void 0) $$bindings.active(active);
  return `${active ? `${validate_component(Badge, "Badge").$$render($$result, Object.assign({}, { color: "green" }, $$props), {}, {
    default: () => {
      return `Active`;
    }
  })}` : `${validate_component(Badge, "Badge").$$render($$result, Object.assign({}, { color: "yellow" }, $$props), {}, {
    default: () => {
      return `Inactive`;
    }
  })}`}`;
});
const ButtonVaultLink = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { tokenVault } = $$props;
  let { chainId } = $$props;
  let { orderbookAddress } = $$props;
  if ($$props.tokenVault === void 0 && $$bindings.tokenVault && tokenVault !== void 0) $$bindings.tokenVault(tokenVault);
  if ($$props.chainId === void 0 && $$bindings.chainId && chainId !== void 0) $$bindings.chainId(chainId);
  if ($$props.orderbookAddress === void 0 && $$bindings.orderbookAddress && orderbookAddress !== void 0) $$bindings.orderbookAddress(orderbookAddress);
  return `<div class="flex cursor-pointer items-center justify-between space-y-2 rounded-lg border border-gray-100 p-2" data-testid="vault-link"><div class="flex flex-col items-start gap-y-2">${validate_component(Tooltip_1, "Tooltip").$$render(
    $$result,
    {
      triggeredBy: `#token-info-${tokenVault.vaultId}`
    },
    {},
    {
      default: () => {
        return `ID: <span class="font-mono">${escape(toHex(tokenVault.vaultId))}</span>`;
      }
    }
  )} <a${add_attribute("href", `/vaults/${chainId}-${orderbookAddress}-${tokenVault.id}`, 0)}${add_attribute("id", `token-info-${tokenVault.vaultId}`, 0)}>${escape(tokenVault.token.name)} (${escape(tokenVault.token.symbol)})</a> <span class="text-sm text-gray-500 dark:text-gray-400">Balance: ${escape(tokenVault.formattedBalance)}</span></div> <div>${slots.buttons ? slots.buttons({}) : ``}</div></div>`;
});
const ButtonTab = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { active = false } = $$props;
  if ($$props.active === void 0 && $$bindings.active && active !== void 0) $$bindings.active(active);
  return `${validate_component(Button, "Button").$$render($$result, Object.assign({}, { disabled: active }, $$props), {}, {
    default: () => {
      return `${slots.default ? slots.default({}) : ``}`;
    }
  })}`;
});
const ChartTimeFilters = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_7_DAYS = TIME_DELTA_24_HOURS * 7;
  const TIME_DELTA_30_DAYS = TIME_DELTA_24_HOURS * 30;
  const TIME_DELTA_1_YEAR = TIME_DELTA_24_HOURS * 365;
  let { timeDelta = TIME_DELTA_1_YEAR } = $$props;
  if ($$props.timeDelta === void 0 && $$bindings.timeDelta && timeDelta !== void 0) $$bindings.timeDelta(timeDelta);
  return `${validate_component(ButtonGroup, "ButtonGroup").$$render(
    $$result,
    {
      class: "bg-gray-800",
      "data-testid": "lightweightChartYearButtons"
    },
    {},
    {
      default: () => {
        return `${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_1_YEAR,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `1 Year`;
            }
          }
        )} ${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_30_DAYS,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `30 Days`;
            }
          }
        )} ${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_7_DAYS,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `7 Days`;
            }
          }
        )} ${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_24_HOURS,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `24 Hours`;
            }
          }
        )}`;
      }
    }
  )}`;
});
const LightweightChart = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$unsubscribe_lightweightChartsTheme;
  let { data = [] } = $$props;
  let { loading = false } = $$props;
  let { emptyMessage = "None found" } = $$props;
  let { title = void 0 } = $$props;
  let { priceSymbol = void 0 } = $$props;
  let { createSeries } = $$props;
  let { lightweightChartsTheme: lightweightChartsTheme2 } = $$props;
  $$unsubscribe_lightweightChartsTheme = subscribe(lightweightChartsTheme2, (value) => value);
  let chartElement = void 0;
  let timeDelta;
  onDestroy(() => {
  });
  if ($$props.data === void 0 && $$bindings.data && data !== void 0) $$bindings.data(data);
  if ($$props.loading === void 0 && $$bindings.loading && loading !== void 0) $$bindings.loading(loading);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  if ($$props.title === void 0 && $$bindings.title && title !== void 0) $$bindings.title(title);
  if ($$props.priceSymbol === void 0 && $$bindings.priceSymbol && priceSymbol !== void 0) $$bindings.priceSymbol(priceSymbol);
  if ($$props.createSeries === void 0 && $$bindings.createSeries && createSeries !== void 0) $$bindings.createSeries(createSeries);
  if ($$props.lightweightChartsTheme === void 0 && $$bindings.lightweightChartsTheme && lightweightChartsTheme2 !== void 0) $$bindings.lightweightChartsTheme(lightweightChartsTheme2);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `<div class="flex h-full flex-col overflow-hidden rounded-lg border bg-gray-50 dark:border-none dark:bg-gray-700"><div class="flex w-full justify-between border-b p-3 pb-0 dark:border-gray-700"><div class="text-gray-900 dark:text-white">${title !== void 0 ? `<div data-testid="lightweightChartTitle" class="mb-2 text-xl">${escape(title)}</div>` : ``}</div> <div>${loading ? `${validate_component(Spinner, "Spinner").$$render(
      $$result,
      {
        "data-testid": "lightweightChartSpinner",
        class: "mr-2 h-4 w-4",
        color: "white"
      },
      {},
      {}
    )}` : ``} ${data.length > 0 ? `${validate_component(ChartTimeFilters, "ChartTimeFilters").$$render(
      $$result,
      { timeDelta },
      {
        timeDelta: ($$value) => {
          timeDelta = $$value;
          $$settled = false;
        }
      },
      {}
    )}` : ``}</div></div> <div class="relative flex w-full grow items-center justify-center bg-white dark:bg-gray-800">${data.length === 0 && !loading ? `<div class="text-gray-800 dark:text-gray-400" data-testid="lightweightChartEmptyMessage">${escape(emptyMessage)}</div>` : `<div${spread(
      [
        { class: "h-full w-full overflow-hidden" },
        escape_object($$props),
        { "data-testid": "lightweightChartElement" }
      ],
      {}
    )}${add_attribute("this", chartElement, 0)}></div>`}</div></div>`;
  } while (!$$settled);
  $$unsubscribe_lightweightChartsTheme();
  return $$rendered;
});
const deduplicateByTime = (data) => {
  const uniqueData = [];
  const seenTimes = /* @__PURE__ */ new Set();
  for (const dataPoint of data) {
    if (!seenTimes.has(dataPoint.time)) {
      uniqueData.push(dataPoint);
      seenTimes.add(dataPoint.time);
    }
  }
  return uniqueData;
};
const transformAndSortData = (data, options) => {
  const { valueTransform, timeTransform } = options;
  const transformedData = data.map((d) => ({
    value: valueTransform(d),
    time: timeTransform(d)
  }));
  const sortedData = sortBy(transformedData, (d) => d.time);
  return deduplicateByTime(sortedData);
};
const TanstackLightweightChartLine = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let data;
  let $query, $$unsubscribe_query;
  let { query } = $$props;
  $$unsubscribe_query = subscribe(query, (value) => $query = value);
  let { timeTransform } = $$props;
  let { valueTransform } = $$props;
  let { lightweightChartsTheme: lightweightChartsTheme2 } = $$props;
  const createSeries = (chart) => chart.addLineSeries({ lineWidth: 1 });
  if ($$props.query === void 0 && $$bindings.query && query !== void 0) $$bindings.query(query);
  if ($$props.timeTransform === void 0 && $$bindings.timeTransform && timeTransform !== void 0) $$bindings.timeTransform(timeTransform);
  if ($$props.valueTransform === void 0 && $$bindings.valueTransform && valueTransform !== void 0) $$bindings.valueTransform(valueTransform);
  if ($$props.lightweightChartsTheme === void 0 && $$bindings.lightweightChartsTheme && lightweightChartsTheme2 !== void 0) $$bindings.lightweightChartsTheme(lightweightChartsTheme2);
  data = transformAndSortData($query.data ?? [], { valueTransform, timeTransform });
  $$unsubscribe_query();
  return `${validate_component(LightweightChart, "LightweightChart").$$render($$result, Object.assign({}, { createSeries }, { data }, { loading: $query.isLoading }, $$props, { lightweightChartsTheme: lightweightChartsTheme2 }), {}, {})}`;
});
function prepareHistoricalOrderChartData(takeOrderEntities, colorTheme2) {
  const transformedData = takeOrderEntities.map((d) => ({
    value: Math.abs(Number(d.inputVaultBalanceChange.formattedAmount) / Number(d.outputVaultBalanceChange.formattedAmount)),
    time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
    color: colorTheme2 == "dark" ? "#5178FF" : "#4E4AF6",
    outputAmount: Number(d.outputVaultBalanceChange.amount)
  }));
  const uniqueTimestamps = Array.from(new Set(transformedData.map((d) => d.time)));
  const finalData = [];
  uniqueTimestamps.forEach((timestamp) => {
    const objectsWithSameTimestamp = transformedData.filter((d) => d.time === timestamp);
    if (objectsWithSameTimestamp.length > 1) {
      const ioratioSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.value * d.outputAmount, 0);
      const outputAmountSum = objectsWithSameTimestamp.reduce((acc, d) => acc + d.outputAmount, 0);
      const ioratioAverage = ioratioSum / outputAmountSum;
      finalData.push({
        value: ioratioAverage,
        time: timestamp,
        color: objectsWithSameTimestamp[0].color
      });
    } else {
      finalData.push(objectsWithSameTimestamp[0]);
    }
  });
  return sortBy(finalData, (d) => d.time);
}
const OrderTradesChart = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let query;
  let $colorTheme, $$unsubscribe_colorTheme;
  let { order } = $$props;
  let { colorTheme: colorTheme2 } = $$props;
  $$unsubscribe_colorTheme = subscribe(colorTheme2, (value) => $colorTheme = value);
  let { lightweightChartsTheme: lightweightChartsTheme2 } = $$props;
  if ($$props.order === void 0 && $$bindings.order && order !== void 0) $$bindings.order(order);
  if ($$props.colorTheme === void 0 && $$bindings.colorTheme && colorTheme2 !== void 0) $$bindings.colorTheme(colorTheme2);
  if ($$props.lightweightChartsTheme === void 0 && $$bindings.lightweightChartsTheme && lightweightChartsTheme2 !== void 0) $$bindings.lightweightChartsTheme(lightweightChartsTheme2);
  query = createQuery({
    queryKey: [QKEY_ORDER_TRADES_LIST, order.id],
    queryFn: async () => {
      const data = await order.getTradesList(BigInt(1e3), void 0, 1);
      if (data.error) throw new Error(data.error.readableMsg);
      return prepareHistoricalOrderChartData(data.value, $colorTheme);
    }
  });
  $$unsubscribe_colorTheme();
  return `${validate_component(TanstackLightweightChartLine, "TanstackLightweightChartLine").$$render(
    $$result,
    {
      title: "Trades",
      query,
      timeTransform: (d) => d.time,
      valueTransform: (d) => d.value,
      emptyMessage: "No trades found",
      lightweightChartsTheme: lightweightChartsTheme2
    },
    {},
    {}
  )}`;
});
const TableTimeFilters = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  const TIME_DELTA_24_HOURS = 60 * 60 * 24;
  const TIME_DELTA_48_HOURS = TIME_DELTA_24_HOURS * 2;
  let timeDelta;
  let { startTimestamp } = $$props;
  let { endTimestamp } = $$props;
  if ($$props.startTimestamp === void 0 && $$bindings.startTimestamp && startTimestamp !== void 0) $$bindings.startTimestamp(startTimestamp);
  if ($$props.endTimestamp === void 0 && $$bindings.endTimestamp && endTimestamp !== void 0) $$bindings.endTimestamp(endTimestamp);
  return `${validate_component(ButtonGroup, "ButtonGroup").$$render(
    $$result,
    {
      class: "bg-gray-800",
      "data-testid": "lightweightChartYearButtons"
    },
    {},
    {
      default: () => {
        return `${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === void 0,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `All Time`;
            }
          }
        )} ${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_48_HOURS,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `48 Hours`;
            }
          }
        )} ${validate_component(ButtonTab, "ButtonTab").$$render(
          $$result,
          {
            active: timeDelta === TIME_DELTA_24_HOURS,
            size: "xs",
            class: "px-2 py-1"
          },
          {},
          {
            default: () => {
              return `24 Hours`;
            }
          }
        )}`;
      }
    }
  )}`;
});
const OrderTradesListTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let orderTradesQuery;
  let { order } = $$props;
  let { rpcs = void 0 } = $$props;
  let { handleDebugTradeModal: handleDebugTradeModal2 = void 0 } = $$props;
  let startTimestamp;
  let endTimestamp;
  let tradesCount;
  const AppTable = TanstackAppTable;
  if ($$props.order === void 0 && $$bindings.order && order !== void 0) $$bindings.order(order);
  if ($$props.rpcs === void 0 && $$bindings.rpcs && rpcs !== void 0) $$bindings.rpcs(rpcs);
  if ($$props.handleDebugTradeModal === void 0 && $$bindings.handleDebugTradeModal && handleDebugTradeModal2 !== void 0) $$bindings.handleDebugTradeModal(handleDebugTradeModal2);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    orderTradesQuery = createInfiniteQuery({
      queryKey: [order.id, QKEY_ORDER_TRADES_LIST + order.id],
      queryFn: async ({ pageParam }) => {
        tradesCount = void 0;
        const [countResult, tradesResult] = await Promise.all([
          order.getTradeCount(startTimestamp ? BigInt(startTimestamp) : void 0, endTimestamp ? BigInt(endTimestamp) : void 0),
          order.getTradesList(startTimestamp ? BigInt(startTimestamp) : void 0, endTimestamp ? BigInt(endTimestamp) : void 0, pageParam + 1)
        ]);
        if (countResult.error) throw new Error(countResult.error.readableMsg);
        if (tradesResult.error) throw new Error(tradesResult.error.readableMsg);
        const count = countResult.value;
        const trades = tradesResult.value;
        if (typeof count === "number") {
          tradesCount = count;
        }
        return trades;
      },
      initialPageParam: 0,
      getNextPageParam: (lastPage, _allPages, lastPageParam) => {
        return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : void 0;
      }
    });
    $$rendered = `${validate_component(AppTable, "AppTable").$$render(
      $$result,
      {
        query: orderTradesQuery,
        emptyMessage: "No trades found",
        rowHoverable: false,
        queryKey: order.id
      },
      {},
      {
        bodyRow: ({ item }) => {
          return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "px-4 py-2" }, {}, {
            default: () => {
              return `${escape(formatTimestampSecondsAsLocal(BigInt(item.timestamp)))}`;
            }
          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all py-2 min-w-32" }, {}, {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Wallet,
                  value: item.transaction.from
                },
                {},
                {}
              )}`;
            }
          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all py-2 min-w-32" }, {}, {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Transaction,
                  value: item.transaction.id
                },
                {},
                {}
              )}`;
            }
          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all py-2" }, {}, {
            default: () => {
              return `${escape(item.inputVaultBalanceChange.formattedAmount)} ${escape(item.inputVaultBalanceChange.token.symbol)}`;
            }
          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all py-2" }, {}, {
            default: () => {
              return `${escape(item.outputVaultBalanceChange.formattedAmount)} ${escape(item.outputVaultBalanceChange.token.symbol)}`;
            }
          })} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
            $$result,
            {
              tdClass: "break-all py-2",
              "data-testid": "io-ratio"
            },
            {},
            {
              default: () => {
                return `${escape(Math.abs(Number(item.inputVaultBalanceChange.formattedAmount) / Number(item.outputVaultBalanceChange.formattedAmount)))} <span class="text-gray-400">(${escape(Math.abs(Number(item.outputVaultBalanceChange.formattedAmount) / Number(item.inputVaultBalanceChange.formattedAmount)))})</span>`;
              }
            }
          )} ${rpcs && handleDebugTradeModal2 ? `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "py-2" }, {}, {
            default: () => {
              return `<button data-testid="debug-trade-button" class="text-gray-500 hover:text-gray-700">${validate_component(BugOutline, "BugOutline").$$render($$result, { size: "xs" }, {}, {})}</button>`;
            }
          })}` : ``} `;
        },
        head: () => {
          return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-4" }, {}, {
            default: () => {
              return `Date`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
            default: () => {
              return `Sender`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
            default: () => {
              return `Transaction Hash`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
            default: () => {
              return `Input`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
            default: () => {
              return `Output`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
            default: () => {
              return `IO Ratio`;
            }
          })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {})} `;
        },
        timeFilter: () => {
          return `${validate_component(TableTimeFilters, "TableTimeFilters").$$render(
            $$result,
            { startTimestamp, endTimestamp },
            {
              startTimestamp: ($$value) => {
                startTimestamp = $$value;
                $$settled = false;
              },
              endTimestamp: ($$value) => {
                endTimestamp = $$value;
                $$settled = false;
              }
            },
            {}
          )} `;
        },
        info: () => {
          return `${tradesCount !== void 0 ? `<div class="px-2">${escape(tradesCount)} Trades</div>` : ``}`;
        }
      }
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const EditableSpan = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { displayValue } = $$props;
  let textContent;
  let editableSpan;
  createEventDispatcher();
  if ($$props.displayValue === void 0 && $$bindings.displayValue && displayValue !== void 0) $$bindings.displayValue(displayValue);
  return ` <div data-testid="editableSpanWrapper" class="flex items-center gap-x-1 border-b-2 border-dotted text-sm text-gray-400 dark:text-gray-400">${validate_component(PenSolid, "PenSolid").$$render($$result, { class: "h-3 w-3 cursor-pointer" }, {}, {})} <span data-svelte-h="svelte-1hafvgr">Block:</span> <div class="relative"><span data-testid="editableSpan" contenteditable="true"${add_attribute("this", editableSpan, 0)}>${(($$value) => $$value === void 0 ? `${escape(displayValue)}` : $$value)(textContent)}</span></div></div>`;
});
const TanstackOrderQuote = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let orderQuoteQuery;
  let $orderQuoteQuery, $$unsubscribe_orderQuoteQuery = noop, $$subscribe_orderQuoteQuery = () => ($$unsubscribe_orderQuoteQuery(), $$unsubscribe_orderQuoteQuery = subscribe(orderQuoteQuery, ($$value) => $orderQuoteQuery = $$value), orderQuoteQuery);
  let { order } = $$props;
  let { handleQuoteDebugModal: handleQuoteDebugModal2 = void 0 } = $$props;
  let enabled = true;
  useQueryClient();
  useToasts();
  let blockNumber;
  if ($$props.order === void 0 && $$bindings.order && order !== void 0) $$bindings.order(order);
  if ($$props.handleQuoteDebugModal === void 0 && $$bindings.handleQuoteDebugModal && handleQuoteDebugModal2 !== void 0) $$bindings.handleQuoteDebugModal(handleQuoteDebugModal2);
  $$subscribe_orderQuoteQuery(orderQuoteQuery = createQuery({
    queryKey: [order.id, QKEY_ORDER_QUOTE + order.id],
    queryFn: async () => {
      const result = await order.getQuotes(blockNumber);
      if (result.error) throw new Error(result.error.msg);
      return result.value;
    },
    enabled: !!order.id && enabled
  }));
  $$unsubscribe_orderQuoteQuery();
  return `<div class="mt-4"><div class="mb-4 flex items-center justify-between"><h2 class="text-lg font-semibold" data-svelte-h="svelte-i1edlq">Order quotes</h2> <div class="flex items-center gap-x-1">${$orderQuoteQuery.data && $orderQuoteQuery.data.length > 0 && isHex($orderQuoteQuery.data[0].blockNumber) ? `${validate_component(EditableSpan, "EditableSpan").$$render(
    $$result,
    {
      displayValue: hexToNumber($orderQuoteQuery.data[0].blockNumber).toString()
    },
    {},
    {}
  )}` : ``} <span></span> ${validate_component(Refresh, "Refresh").$$render(
    $$result,
    {
      "data-testid": "refresh-button",
      class: "h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400",
      spin: $orderQuoteQuery.isLoading || $orderQuoteQuery.isFetching
    },
    {},
    {}
  )} ${validate_component(PauseSolid, "PauseSolid").$$render(
    $$result,
    {
      class: `ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${""}`
    },
    {},
    {}
  )} ${validate_component(PlaySolid, "PlaySolid").$$render(
    $$result,
    {
      class: `ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${"hidden"}`
    },
    {},
    {}
  )}</div></div> ${validate_component(Table, "Table").$$render(
    $$result,
    {
      divClass: "rounded-lg lg:overflow-hidden overflow-auto dark:border-none border"
    },
    {},
    {
      default: () => {
        return `${validate_component(TableHead, "TableHead").$$render($$result, { "data-testid": "head" }, {}, {
          default: () => {
            return `${validate_component(TableHeadCell, "TableHeadCell").$$render(
              $$result,
              {
                class: "w-[80px]",
                "data-testid": "orderQuotesPair"
              },
              {},
              {
                default: () => {
                  return `Pair`;
                }
              }
            )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
              $$result,
              {
                class: "w-1/4",
                "data-testid": "orderQuotesMaxOutput"
              },
              {},
              {
                default: () => {
                  return `Maximum Output`;
                }
              }
            )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
              $$result,
              {
                class: "w-1/4",
                "data-testid": "orderQuotesPrice"
              },
              {},
              {
                default: () => {
                  return `Price`;
                }
              }
            )} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { "data-testid": "orderQuotesPrice" }, {}, {
              default: () => {
                return `Maximum Input`;
              }
            })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { class: "w-[50px]" }, {}, {})}`;
          }
        })} ${validate_component(TableBody, "TableBody").$$render($$result, {}, {}, {
          default: () => {
            return `${$orderQuoteQuery.data && $orderQuoteQuery.data.length > 0 ? `${each($orderQuoteQuery.data, (item, index) => {
              return `${item.success && item.data ? `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, { "data-testid": "bodyRow" }, {}, {
                default: () => {
                  return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${escape(item.pair.pairName)}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${escape(item.data.formattedMaxOutput)}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${escape(item.data.formattedRatio)} <span class="text-gray-400">(${escape(item.data.formattedInverseRatio)})</span>`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${escape(item.data.formattedMaxInput)}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${handleQuoteDebugModal2 ? `<button>${validate_component(BugOutline, "BugOutline").$$render($$result, { size: "sm", color: "grey" }, {}, {})} </button>` : ``} `;
                    }
                  })} `;
                }
              })}` : `${!item.success && item.error ? `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                default: () => {
                  return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${escape(item.pair.pairName)}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
                    $$result,
                    {
                      colspan: "3",
                      class: "text-sm text-red-500 dark:text-red-400"
                    },
                    {},
                    {
                      default: () => {
                        return `${validate_component(Tooltip_1, "Tooltip").$$render(
                          $$result,
                          {
                            triggeredBy: `#quote-error-${index}`,
                            customClass: "max-w-sm whitespace-pre-wrap break-words"
                          },
                          {},
                          {
                            default: () => {
                              return `${escape(item.error)} `;
                            }
                          }
                        )} <div class="flex items-start gap-2"><button type="button" class="mt-0.5 rounded border border-transparent p-1 text-gray-400 transition hover:bg-gray-100 hover:text-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-gray-100" aria-label="Copy quote error">${validate_component(ClipboardOutline, "ClipboardOutline").$$render($$result, { size: "sm" }, {}, {})}</button> <div${add_attribute("id", `quote-error-${index}`, 0)} class="max-w-xl cursor-pointer self-start truncate border-dotted border-red-500 pr-2">${escape(item.error)} </div></div> `;
                      }
                    }
                  )} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, {}, {}, {
                    default: () => {
                      return `${handleQuoteDebugModal2 ? `<button>${validate_component(BugOutline, "BugOutline").$$render($$result, { size: "sm", color: "grey" }, {}, {})} </button>` : ``} `;
                    }
                  })} `;
                }
              })}` : ``}`}`;
            })}` : `${$orderQuoteQuery.isError ? `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
              default: () => {
                return `${validate_component(TableBodyCell, "TableBodyCell").$$render(
                  $$result,
                  {
                    colspan: "3",
                    class: "text-center text-red-500 dark:text-red-400"
                  },
                  {},
                  {
                    default: () => {
                      return `${escape("Error fetching quotes:")} <br> ${escape($orderQuoteQuery.error)}`;
                    }
                  }
                )}`;
              }
            })}` : ``}`}`;
          }
        })}`;
      }
    }
  )}</div>`;
});
let codeMirrorDisabled = true;
const OrderDetail = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let orderDetailQuery;
  let $codeMirrorTheme, $$unsubscribe_codeMirrorTheme;
  let $orderDetailQuery, $$unsubscribe_orderDetailQuery = noop, $$subscribe_orderDetailQuery = () => ($$unsubscribe_orderDetailQuery(), $$unsubscribe_orderDetailQuery = subscribe(orderDetailQuery, ($$value) => $orderDetailQuery = $$value), orderDetailQuery);
  let { handleQuoteDebugModal: handleQuoteDebugModal2 = void 0 } = $$props;
  let { handleDebugTradeModal: handleDebugTradeModal2 = void 0 } = $$props;
  let { colorTheme: colorTheme2 } = $$props;
  let { codeMirrorTheme: codeMirrorTheme2 } = $$props;
  $$unsubscribe_codeMirrorTheme = subscribe(codeMirrorTheme2, (value) => $codeMirrorTheme = value);
  let { lightweightChartsTheme: lightweightChartsTheme2 } = $$props;
  let { orderbookAddress } = $$props;
  let { orderHash } = $$props;
  let { chainId } = $$props;
  let { rpcs = void 0 } = $$props;
  let { onRemove } = $$props;
  let { onDeposit } = $$props;
  let { onWithdraw } = $$props;
  let { onWithdrawAll = void 0 } = $$props;
  let codeMirrorStyles = {};
  const queryClient = useQueryClient();
  const { matchesAccount } = useAccount();
  useToasts();
  const raindexClient = useRaindexClient();
  const interval = setInterval(
    async () => {
      await invalidateTanstackQueries(queryClient, [orderHash]);
    },
    1e4
  );
  onDestroy(() => {
    clearInterval(interval);
  });
  const vaultTypesMap = [
    {
      key: "Output vaults",
      type: "output",
      getter: "outputsList"
    },
    {
      key: "Input vaults",
      type: "input",
      getter: "inputsList"
    },
    {
      key: "Input & output vaults",
      type: "inputOutput",
      getter: "inputsOutputsList"
    }
  ];
  if ($$props.handleQuoteDebugModal === void 0 && $$bindings.handleQuoteDebugModal && handleQuoteDebugModal2 !== void 0) $$bindings.handleQuoteDebugModal(handleQuoteDebugModal2);
  if ($$props.handleDebugTradeModal === void 0 && $$bindings.handleDebugTradeModal && handleDebugTradeModal2 !== void 0) $$bindings.handleDebugTradeModal(handleDebugTradeModal2);
  if ($$props.colorTheme === void 0 && $$bindings.colorTheme && colorTheme2 !== void 0) $$bindings.colorTheme(colorTheme2);
  if ($$props.codeMirrorTheme === void 0 && $$bindings.codeMirrorTheme && codeMirrorTheme2 !== void 0) $$bindings.codeMirrorTheme(codeMirrorTheme2);
  if ($$props.lightweightChartsTheme === void 0 && $$bindings.lightweightChartsTheme && lightweightChartsTheme2 !== void 0) $$bindings.lightweightChartsTheme(lightweightChartsTheme2);
  if ($$props.orderbookAddress === void 0 && $$bindings.orderbookAddress && orderbookAddress !== void 0) $$bindings.orderbookAddress(orderbookAddress);
  if ($$props.orderHash === void 0 && $$bindings.orderHash && orderHash !== void 0) $$bindings.orderHash(orderHash);
  if ($$props.chainId === void 0 && $$bindings.chainId && chainId !== void 0) $$bindings.chainId(chainId);
  if ($$props.rpcs === void 0 && $$bindings.rpcs && rpcs !== void 0) $$bindings.rpcs(rpcs);
  if ($$props.onRemove === void 0 && $$bindings.onRemove && onRemove !== void 0) $$bindings.onRemove(onRemove);
  if ($$props.onDeposit === void 0 && $$bindings.onDeposit && onDeposit !== void 0) $$bindings.onDeposit(onDeposit);
  if ($$props.onWithdraw === void 0 && $$bindings.onWithdraw && onWithdraw !== void 0) $$bindings.onWithdraw(onWithdraw);
  if ($$props.onWithdrawAll === void 0 && $$bindings.onWithdrawAll && onWithdrawAll !== void 0) $$bindings.onWithdrawAll(onWithdrawAll);
  $$subscribe_orderDetailQuery(orderDetailQuery = createQuery({
    queryKey: [orderHash, QKEY_ORDER + orderHash],
    queryFn: async () => {
      const result = await raindexClient.getOrderByHash(chainId, orderbookAddress, orderHash);
      if (result.error) throw new Error(result.error.readableMsg);
      return result.value;
    }
  }));
  $$unsubscribe_codeMirrorTheme();
  $$unsubscribe_orderDetailQuery();
  return `${validate_component(TanstackPageContentDetail, "TanstackPageContentDetail").$$render(
    $$result,
    {
      query: orderDetailQuery,
      emptyMessage: "Order not found"
    },
    {},
    {
      below: ({ data }) => {
        return `${validate_component(TanstackOrderQuote, "TanstackOrderQuote").$$render($$result, { order: data, handleQuoteDebugModal: handleQuoteDebugModal2 }, {}, {})} ${validate_component(Tabs, "Tabs").$$render(
          $$result,
          {
            style: "underline",
            contentClass: "mt-4",
            defaultClass: "flex flex-wrap space-x-2 rtl:space-x-reverse mt-4 list-none"
          },
          {},
          {
            default: () => {
              return `${validate_component(TabItem, "TabItem").$$render($$result, { title: "Rainlang source" }, {}, {
                default: () => {
                  return `<div class="mb-8 overflow-hidden rounded-lg border dark:border-none">${validate_component(CodeMirrorRainlang, "CodeMirrorRainlang").$$render(
                    $$result,
                    {
                      order: data,
                      codeMirrorTheme: $codeMirrorTheme,
                      codeMirrorDisabled,
                      codeMirrorStyles
                    },
                    {},
                    {}
                  )}</div>`;
                }
              })} ${validate_component(TabItem, "TabItem").$$render($$result, { open: true, title: "Trades" }, {}, {
                default: () => {
                  return `${validate_component(OrderTradesListTable, "OrderTradesListTable").$$render($$result, { order: data, handleDebugTradeModal: handleDebugTradeModal2, rpcs }, {}, {})}`;
                }
              })} ${validate_component(TabItem, "TabItem").$$render($$result, { title: "Volume" }, {}, {
                default: () => {
                  return `<div data-svelte-h="svelte-up4rgn">TODO: Issue #1989</div> `;
                }
              })} ${validate_component(TabItem, "TabItem").$$render($$result, { title: "APY" }, {}, {
                default: () => {
                  return `<div data-svelte-h="svelte-up4rgn">TODO: Issue #1989</div> `;
                }
              })}`;
            }
          }
        )} `;
      },
      chart: ({ data }) => {
        return `${validate_component(OrderTradesChart, "OrderTradesChart").$$render(
          $$result,
          {
            order: data,
            lightweightChartsTheme: lightweightChartsTheme2,
            colorTheme: colorTheme2
          },
          {},
          {}
        )} `;
      },
      card: ({ data }) => {
        return `<div class="flex flex-col gap-y-6">${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
          value: () => {
            return `${validate_component(Hash, "Hash").$$render(
              $$result,
              {
                type: HashType.Identifier,
                shorten: false,
                value: data.orderbook
              },
              {},
              {}
            )} `;
          },
          key: () => {
            return `Orderbook`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
          value: () => {
            let explorerLink = getExplorerLink(data.owner, chainId, "address");
            return `${explorerLink ? `<a${add_attribute("href", explorerLink, 0)} target="_blank" rel="noopener noreferrer" class="flex items-center justify-start space-x-2 text-left text-blue-500 hover:underline">${validate_component(WalletOutline, "WalletOutline").$$render($$result, { size: "sm" }, {}, {})} <span>${escape(data.owner)}</span></a>` : `${validate_component(Hash, "Hash").$$render(
              $$result,
              {
                type: HashType.Wallet,
                shorten: false,
                value: data.owner
              },
              {},
              {}
            )}`} `;
          },
          key: () => {
            return `Owner`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
          value: () => {
            return `${escape(formatTimestampSecondsAsLocal(data.timestampAdded))} `;
          },
          key: () => {
            return `Created`;
          }
        })} ${each(vaultTypesMap, ({ key, type, getter }) => {
          let filteredVaults = data.vaultsList.items.filter((vault) => vault.vaultType === type), vaultsListByType = data[getter];
          return `  ${filteredVaults.length !== 0 ? `${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
            value: () => {
              return `<div class="mt-2 space-y-2">${each(filteredVaults, (vault) => {
                return `${validate_component(ButtonVaultLink, "ButtonVaultLink").$$render(
                  $$result,
                  {
                    tokenVault: vault,
                    chainId,
                    orderbookAddress
                  },
                  {},
                  {
                    buttons: () => {
                      return `${matchesAccount(vault.owner) ? `<div class="flex gap-1">${validate_component(Button, "Button").$$render(
                        $$result,
                        {
                          color: "light",
                          size: "xs",
                          "data-testid": "deposit-button"
                        },
                        {},
                        {
                          default: () => {
                            return `${validate_component(ArrowDownToBracketOutline, "ArrowDownToBracketOutline").$$render($$result, { size: "xs" }, {}, {})} `;
                          }
                        }
                      )} ${validate_component(Button, "Button").$$render(
                        $$result,
                        {
                          color: "light",
                          size: "xs",
                          "data-testid": "withdraw-button"
                        },
                        {},
                        {
                          default: () => {
                            return `${validate_component(ArrowUpFromBracketOutline, "ArrowUpFromBracketOutline").$$render($$result, { size: "xs" }, {}, {})} `;
                          }
                        }
                      )} </div>` : ``} `;
                    }
                  }
                )}`;
              })}</div> `;
            },
            key: () => {
              return `<div class="flex items-center justify-between"><div class="flex items-center gap-x-2">${escape(key)} ${type === "inputOutput" ? `${validate_component(InfoCircleOutline, "InfoCircleOutline").$$render($$result, { class: "h-4 w-4" }, {}, {})}${validate_component(Tooltip, "Tooltip").$$render($$result, {}, {}, {
                default: () => {
                  return `${escape("These vaults can be an input or an output for this order")}`;
                }
              })}` : ``}</div> ${onWithdrawAll ? `${validate_component(Button, "Button").$$render(
                $$result,
                {
                  color: "light",
                  size: "xs",
                  disabled: !vaultsListByType || vaultsListByType.getWithdrawableVaults()?.value?.length === 0,
                  "data-testid": "withdraw-all-button"
                },
                {},
                {
                  default: () => {
                    return `${validate_component(ArrowUpFromBracketOutline, "ArrowUpFromBracketOutline").$$render($$result, { size: "xs", class: "mr-2" }, {}, {})}
										Withdraw all
									`;
                  }
                }
              )}` : ``}</div> `;
            }
          })}` : ``}`;
        })} ${onWithdrawAll ? `${validate_component(Button, "Button").$$render(
          $$result,
          {
            size: "xs",
            disabled: !$orderDetailQuery.data?.vaultsList || $orderDetailQuery.data?.vaultsList?.getWithdrawableVaults()?.value?.length === 0,
            "data-testid": "withdraw-all-button"
          },
          {},
          {
            default: () => {
              return `${validate_component(ArrowUpFromBracketOutline, "ArrowUpFromBracketOutline").$$render($$result, { size: "xs", class: "mr-2" }, {}, {})}
					Withdraw all vaults`;
            }
          }
        )}` : ``}</div> `;
      },
      top: ({ data }) => {
        return `<div class="flex w-full flex-wrap items-center justify-between gap-4 text-3xl font-medium lg:justify-between dark:text-white"><div class="flex items-center gap-x-2"><div class="flex gap-x-2"><span class="font-light" data-svelte-h="svelte-18smzpp">Order</span> ${validate_component(Hash, "Hash").$$render($$result, { shorten: true, value: data.orderHash }, {}, {})}</div> ${validate_component(BadgeActive, "BadgeActive").$$render($$result, { active: data.active, large: true }, {}, {})}</div> <div class="flex items-center gap-2">${matchesAccount(data.owner) ? `${data.active ? `${validate_component(Button, "Button").$$render(
          $$result,
          {
            "data-testid": "remove-button",
            "aria-label": "Remove order"
          },
          {},
          {
            default: () => {
              return `Remove`;
            }
          }
        )}` : ``}` : ``} ${validate_component(Refresh, "Refresh").$$render(
          $$result,
          {
            testId: "top-refresh",
            spin: $orderDetailQuery.isLoading || $orderDetailQuery.isFetching
          },
          {},
          {}
        )}</div></div>`;
      }
    }
  )}`;
});
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  const context = getAllContexts();
  const raindexClient = useRaindexClient();
  const queryClient = useQueryClient();
  const { chainId, orderbook, orderHash } = $page.params;
  const parsedOrderHash = orderHash;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook;
  let rpcs = [];
  function onRemove(_raindexClient, order) {
    handleOrderRemoveModal(
      order,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context
    );
  }
  function onDeposit(_raindexClient, vault) {
    handleDepositModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context
    );
  }
  function onWithdraw(_raindexClient, vault) {
    handleWithdrawModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [parsedOrderHash]);
      },
      context
    );
  }
  {
    if (raindexClient) {
      const networks = raindexClient.getNetworkByChainId(parsedChainId);
      if (networks.error) throw new Error(networks.error.readableMsg);
      rpcs = networks.value.rpcs;
    }
  }
  $$unsubscribe_page();
  return `${validate_component(PageHeader, "PageHeader").$$render(
    $$result,
    {
      title: "Order",
      pathname: $page.url.pathname
    },
    {},
    {}
  )} <div data-testid="order-detail">${validate_component(OrderDetail, "OrderDetail").$$render(
    $$result,
    {
      chainId: parsedChainId,
      orderbookAddress,
      orderHash: parsedOrderHash,
      colorTheme,
      codeMirrorTheme,
      lightweightChartsTheme,
      handleQuoteDebugModal,
      handleDebugTradeModal,
      onRemove,
      onDeposit,
      onWithdraw,
      rpcs
    },
    {},
    {}
  )}</div>`;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
