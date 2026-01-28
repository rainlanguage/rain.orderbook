import { c as create_ssr_component, a as compute_rest_props, g as getContext, v as validate_component, b as spread, e as escape_attribute_value, d as escape_object, i as compute_slots, f as add_attribute, l as createEventDispatcher, h as escape, j as each, k as subscribe } from "./ssr.js";
import { i as is_void, B as Button } from "./darkMode.js";
import { W as Wrapper, L as Label, I as Input, g as getNetworkName } from "./order.js";
import { twMerge } from "tailwind-merge";
import { u as useRaindexClient, T as Tooltip_1 } from "./queryClient.js";
import { l as labelClass, i as inputClass, C as ChevronDownSolid, D as Dropdown } from "./ChevronDownSolid.js";
import { isEmpty } from "lodash";
import { p as page } from "./stores.js";
import { A as Alert } from "./sentry.js";
import { u as useAccount } from "./modal.js";
const DropdownItem = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let active;
  let liClass;
  let $$restProps = compute_rest_props($$props, ["defaultClass", "href", "activeClass"]);
  let { defaultClass = "font-medium py-2 px-4 text-sm hover:bg-gray-100 dark:hover:bg-gray-600" } = $$props;
  let { href = void 0 } = $$props;
  let { activeClass = void 0 } = $$props;
  const context = getContext("DropdownType") ?? {};
  const activeUrlStore = getContext("activeUrl");
  let sidebarUrl = "";
  activeUrlStore.subscribe((value) => {
    sidebarUrl = value;
  });
  let wrap = true;
  function init(node) {
    wrap = node.parentElement?.tagName === "UL";
  }
  if ($$props.defaultClass === void 0 && $$bindings.defaultClass && defaultClass !== void 0) $$bindings.defaultClass(defaultClass);
  if ($$props.href === void 0 && $$bindings.href && href !== void 0) $$bindings.href(href);
  if ($$props.activeClass === void 0 && $$bindings.activeClass && activeClass !== void 0) $$bindings.activeClass(activeClass);
  active = sidebarUrl ? href === sidebarUrl : false;
  liClass = twMerge(defaultClass, href ? "block" : "w-full text-left", active && (activeClass ?? context.activeClass), $$props.class);
  return `${validate_component(Wrapper, "Wrapper").$$render($$result, { tag: "li", show: wrap, use: init }, {}, {
    default: () => {
      return `${((tag) => {
        return tag ? `<${href ? "a" : "button"}${spread(
          [
            { href: escape_attribute_value(href) },
            {
              type: escape_attribute_value(href ? void 0 : "button")
            },
            {
              role: escape_attribute_value(href ? "link" : "button")
            },
            escape_object($$restProps),
            { class: escape_attribute_value(liClass) }
          ],
          {}
        )}>${is_void(tag) ? "" : `${slots.default ? slots.default({}) : ``}`}${is_void(tag) ? "" : `</${tag}>`}` : "";
      })(href ? "a" : "button")}`;
    }
  })} `;
});
const Checkbox = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["color", "custom", "inline", "group", "value", "checked", "spacing"]);
  let $$slots = compute_slots(slots);
  let { color = "primary" } = $$props;
  let { custom = false } = $$props;
  let { inline = false } = $$props;
  let { group = [] } = $$props;
  let { value = "on" } = $$props;
  let { checked = void 0 } = $$props;
  let { spacing = "me-2" } = $$props;
  let background = getContext("background");
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.custom === void 0 && $$bindings.custom && custom !== void 0) $$bindings.custom(custom);
  if ($$props.inline === void 0 && $$bindings.inline && inline !== void 0) $$bindings.inline(inline);
  if ($$props.group === void 0 && $$bindings.group && group !== void 0) $$bindings.group(group);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.checked === void 0 && $$bindings.checked && checked !== void 0) $$bindings.checked(checked);
  if ($$props.spacing === void 0 && $$bindings.spacing && spacing !== void 0) $$bindings.spacing(spacing);
  return `${validate_component(Label, "Label").$$render(
    $$result,
    {
      class: labelClass(inline, $$props.class),
      show: $$slots.default
    },
    {},
    {
      default: () => {
        return `<input${spread(
          [
            { type: "checkbox" },
            { value: escape_attribute_value(value) },
            escape_object($$restProps),
            {
              class: escape_attribute_value(inputClass(custom, color, true, background, spacing, $$slots.default || $$props.class))
            }
          ],
          {}
        )}${add_attribute("checked", checked, 1)}> ${slots.default ? slots.default({}) : ``}`;
      }
    }
  )} `;
});
const DotsVerticalOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "role", "strokeLinecap", "ariaLabel"]);
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
  let { ariaLabel = "dots vertical outline" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
  if ($$props.strokeLinecap === void 0 && $$bindings.strokeLinecap && strokeLinecap !== void 0) $$bindings.strokeLinecap(strokeLinecap);
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
      { viewBox: "0 0 4 16" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)} stroke-width="3" d="M1.5 2h.01M1.5 8h.01m-.01 6h.01"></path></svg> `;
});
const SearchSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "search solid" } = $$props;
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
  )}><path fill="currentColor" d="M8 15.5a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15Zm11.707 2.793-4-4a1 1 0 0 0-1.414 1.414l4 4a1 1 0 0 0 1.414-1.414Z"></path></svg> `;
});
const DropdownCheckbox = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let selectedCount;
  let allSelected;
  let buttonText;
  createEventDispatcher();
  let { options = {} } = $$props;
  let { value = {} } = $$props;
  let { label = "Select items" } = $$props;
  let { allLabel = "All items" } = $$props;
  let { showAllLabel = true } = $$props;
  let { emptyMessage = "No items available" } = $$props;
  let { onlyTitle = false } = $$props;
  if ($$props.options === void 0 && $$bindings.options && options !== void 0) $$bindings.options(options);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.label === void 0 && $$bindings.label && label !== void 0) $$bindings.label(label);
  if ($$props.allLabel === void 0 && $$bindings.allLabel && allLabel !== void 0) $$bindings.allLabel(allLabel);
  if ($$props.showAllLabel === void 0 && $$bindings.showAllLabel && showAllLabel !== void 0) $$bindings.showAllLabel(showAllLabel);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  if ($$props.onlyTitle === void 0 && $$bindings.onlyTitle && onlyTitle !== void 0) $$bindings.onlyTitle(onlyTitle);
  selectedCount = Object.keys(value).length;
  allSelected = selectedCount === Object.keys(options).length;
  buttonText = selectedCount === 0 ? "Select items" : allSelected ? allLabel : `${selectedCount} item${selectedCount > 1 ? "s" : ""}`;
  return `${validate_component(Label, "Label").$$render($$result, {}, {}, {
    default: () => {
      return `${escape(label)}`;
    }
  })} <div>${validate_component(Button, "Button").$$render(
    $$result,
    {
      color: "alternative",
      class: "flex w-full justify-between overflow-hidden pl-2 pr-0 text-left",
      "data-testid": "dropdown-checkbox-button"
    },
    {},
    {
      default: () => {
        return `<div class="w-[90px] overflow-hidden text-ellipsis whitespace-nowrap">${escape(buttonText)}</div> ${validate_component(ChevronDownSolid, "ChevronDownSolid").$$render(
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
    {
      class: "w-full min-w-72 py-0",
      "data-testid": "dropdown-checkbox"
    },
    {},
    {
      default: () => {
        return `${isEmpty(options) ? `<div class="ml-2 w-full rounded-lg p-3">${escape(emptyMessage)}</div>` : `${Object.keys(options).length > 1 && showAllLabel ? `${validate_component(Checkbox, "Checkbox").$$render(
          $$result,
          {
            "data-testid": "dropdown-checkbox-option",
            class: "w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600",
            checked: allSelected
          },
          {},
          {
            default: () => {
              return `<div class="ml-2">${escape(allLabel)}</div>`;
            }
          }
        )}` : ``}`} ${each(Object.entries(options), ([key, optionValue]) => {
          return `${validate_component(Checkbox, "Checkbox").$$render(
            $$result,
            {
              "data-testid": "dropdown-checkbox-option",
              class: "w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600",
              checked: key in value
            },
            {},
            {
              default: () => {
                return `<div class="ml-2"><div class="text-sm font-medium">${escape(optionValue)}</div> ${!onlyTitle ? `<div class="text-xs text-gray-500">${escape(key)}</div>` : ``}</div> `;
              }
            }
          )}`;
        })}`;
      }
    }
  )}</div>`;
});
function getTokenDisplayName(token) {
  return token.symbol || token.name || "Unknown Token";
}
const DropdownTokensFilter = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let availableTokens;
  let selectedCount;
  let allSelected;
  let buttonText;
  let sortedFilteredTokens;
  let $$unsubscribe_activeTokens;
  let $tokensQuery, $$unsubscribe_tokensQuery;
  let { tokensQuery } = $$props;
  $$unsubscribe_tokensQuery = subscribe(tokensQuery, (value) => $tokensQuery = value);
  let { activeTokens } = $$props;
  $$unsubscribe_activeTokens = subscribe(activeTokens, (value) => value);
  let { selectedTokens } = $$props;
  let { label = "Filter by tokens" } = $$props;
  let { allLabel = "All tokens" } = $$props;
  let { emptyMessage = "No tokens available" } = $$props;
  let { loadingMessage = "Loading tokens..." } = $$props;
  let filteredTokens = [];
  let searchTerm = "";
  let selectedIndex = 0;
  if ($$props.tokensQuery === void 0 && $$bindings.tokensQuery && tokensQuery !== void 0) $$bindings.tokensQuery(tokensQuery);
  if ($$props.activeTokens === void 0 && $$bindings.activeTokens && activeTokens !== void 0) $$bindings.activeTokens(activeTokens);
  if ($$props.selectedTokens === void 0 && $$bindings.selectedTokens && selectedTokens !== void 0) $$bindings.selectedTokens(selectedTokens);
  if ($$props.label === void 0 && $$bindings.label && label !== void 0) $$bindings.label(label);
  if ($$props.allLabel === void 0 && $$bindings.allLabel && allLabel !== void 0) $$bindings.allLabel(allLabel);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  if ($$props.loadingMessage === void 0 && $$bindings.loadingMessage && loadingMessage !== void 0) $$bindings.loadingMessage(loadingMessage);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    availableTokens = $tokensQuery?.data || [];
    selectedCount = selectedTokens.length;
    allSelected = selectedCount === availableTokens.length && availableTokens.length > 0;
    buttonText = selectedCount === 0 ? "Select tokens" : allSelected ? allLabel : `${selectedCount} token${selectedCount > 1 ? "s" : ""}`;
    {
      {
        let tokensToFilter = availableTokens;
        const getKey = (token) => `${token.address}-${token.chainId}`;
        const uniqueTokensMap = /* @__PURE__ */ new Map();
        tokensToFilter.forEach((token) => {
          const key = getKey(token);
          if (token.address && !uniqueTokensMap.has(key)) {
            uniqueTokensMap.set(key, token);
          }
        });
        const uniqueTokens = Array.from(uniqueTokensMap.values());
        if (searchTerm.trim() === "") {
          filteredTokens = uniqueTokens;
        } else {
          const term = searchTerm.toLowerCase();
          filteredTokens = uniqueTokens.filter((token) => token.symbol?.toLowerCase().includes(term) || token.name?.toLowerCase().includes(term) || token.address?.toLowerCase().includes(term));
          selectedIndex = filteredTokens.length > 0 ? 0 : -1;
        }
      }
    }
    sortedFilteredTokens = [...filteredTokens].sort((a, b) => {
      const aSelected = selectedTokens.includes(a.address);
      const bSelected = selectedTokens.includes(b.address);
      if (aSelected === bSelected) return 0;
      return aSelected ? -1 : 1;
    });
    $$rendered = `<div class="flex flex-col gap-x-2">${validate_component(Label, "Label").$$render($$result, {}, {}, {
      default: () => {
        return `${escape(label)}`;
      }
    })} <div>${validate_component(Button, "Button").$$render(
      $$result,
      {
        color: "alternative",
        class: "flex w-full justify-between overflow-hidden pl-2 pr-0 text-left",
        "data-testid": "dropdown-tokens-filter-button",
        "aria-label": "Select tokens to filter",
        "aria-expanded": "false",
        "aria-haspopup": "listbox"
      },
      {},
      {
        default: () => {
          return `<div class="w-[90px] overflow-hidden text-ellipsis whitespace-nowrap">${escape(buttonText)}</div> ${validate_component(ChevronDownSolid, "ChevronDownSolid").$$render(
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
      {
        class: "max-h-[75vh] w-full min-w-60 overflow-y-auto py-0",
        "data-testid": "dropdown-tokens-filter"
      },
      {},
      {
        default: () => {
          return `${$tokensQuery.isLoading ? `<div class="ml-2 w-full rounded-lg p-3">${escape(loadingMessage)}</div>` : `${$tokensQuery.isError ? `<div class="ml-2 w-full rounded-lg p-3 text-red-500">Cannot load tokens list: ${escape($tokensQuery.error?.message || "Unknown error")}</div>` : `${isEmpty(availableTokens) ? `<div class="ml-2 w-full rounded-lg p-3">${escape(emptyMessage)}</div>` : ` <div class="sticky top-0 bg-white p-3 dark:bg-gray-800">${validate_component(Input, "Input").$$render(
            $$result,
            {
              placeholder: "Search tokens...",
              autofocus: true,
              "data-testid": "tokens-filter-search",
              value: searchTerm
            },
            {
              value: ($$value) => {
                searchTerm = $$value;
                $$settled = false;
              }
            },
            {
              left: () => {
                return `${validate_component(SearchSolid, "SearchSolid").$$render(
                  $$result,
                  {
                    slot: "left",
                    class: "h-4 w-4 text-gray-500"
                  },
                  {},
                  {}
                )}`;
              }
            }
          )}</div> ${isEmpty(filteredTokens) ? `<div class="ml-2 w-full rounded-lg p-3" data-svelte-h="svelte-1nurpe8">No tokens match your search</div>` : `${each(sortedFilteredTokens, (token, index) => {
            return `${validate_component(Checkbox, "Checkbox").$$render(
              $$result,
              {
                "data-testid": "dropdown-tokens-filter-option",
                class: "w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600 " + (selectedIndex === index ? "bg-blue-100 dark:bg-blue-900" : ""),
                checked: !!(token.address && selectedTokens.includes(token.address))
              },
              {},
              {
                default: () => {
                  return `<div class="ml-2 flex w-full"><div class="flex-1 text-sm font-medium">${escape(getTokenDisplayName(token))}</div> <div class="text-xs text-gray-500">${escape(getNetworkName(token.chainId))} </div></div> `;
                }
              }
            )}`;
          })}`}`}`}`}`;
        }
      }
    )}</div></div>`;
  } while (!$$settled);
  $$unsubscribe_activeTokens();
  $$unsubscribe_tokensQuery();
  return $$rendered;
});
function getDisplayName(ob) {
  const truncatedAddr = `${ob.address.slice(0, 6)}...${ob.address.slice(-4)}`;
  return ob.label ? `${ob.label} (${truncatedAddr})` : truncatedAddr;
}
const DropdownOrderbooksFilter = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let orderbooksResult;
  let orderbooksMap;
  let orderbooksError;
  let availableOrderbooks;
  let selectedCount;
  let allSelected;
  let buttonText;
  let sortedFilteredOrderbooks;
  let $$unsubscribe_activeOrderbookAddresses;
  let { activeOrderbookAddresses } = $$props;
  $$unsubscribe_activeOrderbookAddresses = subscribe(activeOrderbookAddresses, (value) => value);
  let { selectedOrderbookAddresses } = $$props;
  let { selectedChainIds } = $$props;
  let { label = "Filter by orderbook" } = $$props;
  let { allLabel = "All orderbooks" } = $$props;
  let { emptyMessage = "No orderbooks available" } = $$props;
  const raindexClient = useRaindexClient();
  let filteredOrderbooks = [];
  let searchTerm = "";
  let selectedIndex = 0;
  if ($$props.activeOrderbookAddresses === void 0 && $$bindings.activeOrderbookAddresses && activeOrderbookAddresses !== void 0) $$bindings.activeOrderbookAddresses(activeOrderbookAddresses);
  if ($$props.selectedOrderbookAddresses === void 0 && $$bindings.selectedOrderbookAddresses && selectedOrderbookAddresses !== void 0) $$bindings.selectedOrderbookAddresses(selectedOrderbookAddresses);
  if ($$props.selectedChainIds === void 0 && $$bindings.selectedChainIds && selectedChainIds !== void 0) $$bindings.selectedChainIds(selectedChainIds);
  if ($$props.label === void 0 && $$bindings.label && label !== void 0) $$bindings.label(label);
  if ($$props.allLabel === void 0 && $$bindings.allLabel && allLabel !== void 0) $$bindings.allLabel(allLabel);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    orderbooksResult = raindexClient.getAllOrderbooks();
    orderbooksMap = orderbooksResult?.value ?? /* @__PURE__ */ new Map();
    orderbooksError = orderbooksResult?.error;
    availableOrderbooks = (() => {
      const items = [];
      orderbooksMap.forEach((cfg, key) => {
        if (selectedChainIds.length === 0 || selectedChainIds.includes(cfg.network.chainId)) {
          items.push({
            key,
            address: cfg.address,
            label: cfg.label,
            chainId: cfg.network.chainId
          });
        }
      });
      return items;
    })();
    selectedCount = selectedOrderbookAddresses.length;
    allSelected = selectedCount === availableOrderbooks.length && availableOrderbooks.length > 0;
    buttonText = selectedCount === 0 ? "Select orderbooks" : allSelected ? allLabel : `${selectedCount} orderbook${selectedCount > 1 ? "s" : ""}`;
    {
      {
        if (searchTerm.trim() === "") {
          filteredOrderbooks = availableOrderbooks;
          selectedIndex = 0;
        } else {
          const term = searchTerm.toLowerCase();
          filteredOrderbooks = availableOrderbooks.filter((ob) => ob.label?.toLowerCase().includes(term) || ob.address?.toLowerCase().includes(term) || ob.key?.toLowerCase().includes(term));
          selectedIndex = filteredOrderbooks.length > 0 ? 0 : -1;
        }
      }
    }
    sortedFilteredOrderbooks = [...filteredOrderbooks].sort((a, b) => {
      const aSelected = selectedOrderbookAddresses.some((addr) => addr.toLowerCase() === a.address.toLowerCase());
      const bSelected = selectedOrderbookAddresses.some((addr) => addr.toLowerCase() === b.address.toLowerCase());
      if (aSelected === bSelected) return 0;
      return aSelected ? -1 : 1;
    });
    $$rendered = `<div class="flex flex-col gap-x-2">${validate_component(Label, "Label").$$render($$result, {}, {}, {
      default: () => {
        return `${escape(label)}`;
      }
    })} <div>${validate_component(Button, "Button").$$render(
      $$result,
      {
        color: "alternative",
        class: "flex w-full justify-between overflow-hidden pl-2 pr-0 text-left",
        "data-testid": "dropdown-orderbooks-filter-button",
        "aria-label": "Select orderbooks to filter",
        "aria-expanded": "false",
        "aria-haspopup": "listbox"
      },
      {},
      {
        default: () => {
          return `<div class="w-[110px] overflow-hidden text-ellipsis whitespace-nowrap">${escape(buttonText)}</div> ${validate_component(ChevronDownSolid, "ChevronDownSolid").$$render(
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
      {
        class: "max-h-[75vh] w-full min-w-60 overflow-y-auto py-0",
        "data-testid": "dropdown-orderbooks-filter"
      },
      {},
      {
        default: () => {
          return `${orderbooksError ? `<div class="ml-2 w-full rounded-lg p-3 text-red-500">Cannot load orderbooks list: ${escape(orderbooksError.readableMsg || "Unknown error")}</div>` : `${isEmpty(availableOrderbooks) ? `<div class="ml-2 w-full rounded-lg p-3">${escape(emptyMessage)}</div>` : `<div class="sticky top-0 bg-white p-3 dark:bg-gray-800">${validate_component(Input, "Input").$$render(
            $$result,
            {
              placeholder: "Search orderbooks...",
              autofocus: true,
              "data-testid": "orderbooks-filter-search",
              value: searchTerm
            },
            {
              value: ($$value) => {
                searchTerm = $$value;
                $$settled = false;
              }
            },
            {
              left: () => {
                return `${validate_component(SearchSolid, "SearchSolid").$$render(
                  $$result,
                  {
                    slot: "left",
                    class: "h-4 w-4 text-gray-500"
                  },
                  {},
                  {}
                )}`;
              }
            }
          )}</div> ${isEmpty(filteredOrderbooks) ? `<div class="ml-2 w-full rounded-lg p-3" data-svelte-h="svelte-1bbh0so">No orderbooks match your search</div>` : `${each(sortedFilteredOrderbooks, (orderbook, index) => {
            return `${validate_component(Checkbox, "Checkbox").$$render(
              $$result,
              {
                "data-testid": "dropdown-orderbooks-filter-option",
                class: "w-full rounded-lg p-3 hover:bg-gray-100 dark:hover:bg-gray-600 " + (selectedIndex === index ? "bg-blue-100 dark:bg-blue-900" : ""),
                checked: !!(orderbook.address && selectedOrderbookAddresses.some((addr) => addr.toLowerCase() === orderbook.address.toLowerCase()))
              },
              {},
              {
                default: () => {
                  return `<div class="ml-2 flex w-full"><div class="flex-1 text-sm font-medium">${escape(getDisplayName(orderbook))}</div> <div class="text-xs text-gray-500">${escape(getNetworkName(orderbook.chainId))} </div></div> `;
                }
              }
            )}`;
          })}`}`}`}`;
        }
      }
    )}</div></div>`;
  } while (!$$settled);
  $$unsubscribe_activeOrderbookAddresses();
  return $$rendered;
});
function getAccountsAsOptions(accounts) {
  if (!accounts)
    return {};
  return Object.fromEntries(Array.from(accounts.entries()).map(([key, value]) => [key, value.address]));
}
const DropdownOrderListAccounts = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let accounts;
  let options;
  let $activeAccountsItems, $$unsubscribe_activeAccountsItems;
  let { activeAccountsItems } = $$props;
  $$unsubscribe_activeAccountsItems = subscribe(activeAccountsItems, (value) => $activeAccountsItems = value);
  const raindexClient = useRaindexClient();
  if ($$props.activeAccountsItems === void 0 && $$bindings.activeAccountsItems && activeAccountsItems !== void 0) $$bindings.activeAccountsItems(activeAccountsItems);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    accounts = raindexClient.getAllAccounts();
    options = getAccountsAsOptions(accounts.value);
    $$rendered = `<div data-testid="accounts-dropdown">${validate_component(DropdownCheckbox, "DropdownCheckbox").$$render(
      $$result,
      {
        options,
        label: "Accounts",
        allLabel: "All accounts",
        emptyMessage: "No accounts added",
        value: $activeAccountsItems
      },
      {
        value: ($$value) => {
          $activeAccountsItems = $$value;
          $$settled = false;
        }
      },
      {}
    )}</div>`;
  } while (!$$settled);
  $$unsubscribe_activeAccountsItems();
  return $$rendered;
});
const CheckboxActiveOrders = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $showInactiveOrders, $$unsubscribe_showInactiveOrders;
  let { showInactiveOrders } = $$props;
  $$unsubscribe_showInactiveOrders = subscribe(showInactiveOrders, (value) => $showInactiveOrders = value);
  let checked = $showInactiveOrders ? true : false;
  if ($$props.showInactiveOrders === void 0 && $$bindings.showInactiveOrders && showInactiveOrders !== void 0) $$bindings.showInactiveOrders(showInactiveOrders);
  $$unsubscribe_showInactiveOrders();
  return `<div data-testid="order-status-checkbox">${validate_component(Checkbox, "Checkbox").$$render($$result, { checked }, {}, {
    default: () => {
      return `Include inactive orders`;
    }
  })}</div>`;
});
const InputOrderHash = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $orderHash, $$unsubscribe_orderHash;
  let { orderHash } = $$props;
  $$unsubscribe_orderHash = subscribe(orderHash, (value2) => $orderHash = value2);
  let { value = $orderHash } = $$props;
  if ($$props.orderHash === void 0 && $$bindings.orderHash && orderHash !== void 0) $$bindings.orderHash(orderHash);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  $$unsubscribe_orderHash();
  return `<div data-testid="order-hash-input" class="flex flex-col gap-x-2">${validate_component(Label, "Label").$$render($$result, {}, {}, {
    default: () => {
      return `Order hash`;
    }
  })} <div class="w-full lg:w-32"><input type="text" placeholder="0x..." class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"${add_attribute("value", value, 0)}></div></div>`;
});
const CheckboxZeroBalanceVault = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $hideZeroBalanceVaults, $$unsubscribe_hideZeroBalanceVaults;
  let { hideZeroBalanceVaults } = $$props;
  $$unsubscribe_hideZeroBalanceVaults = subscribe(hideZeroBalanceVaults, (value) => $hideZeroBalanceVaults = value);
  if ($$props.hideZeroBalanceVaults === void 0 && $$bindings.hideZeroBalanceVaults && hideZeroBalanceVaults !== void 0) $$bindings.hideZeroBalanceVaults(hideZeroBalanceVaults);
  $$unsubscribe_hideZeroBalanceVaults();
  return `<div data-testid="zero-balance-vault-checkbox" class="flex items-center gap-x-2">${validate_component(Label, "Label").$$render(
    $$result,
    {
      for: "hide-empty-vaults",
      class: "cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
    },
    {},
    {
      default: () => {
        return `Hide empty vaults`;
      }
    }
  )} ${validate_component(Checkbox, "Checkbox").$$render(
    $$result,
    {
      id: "hide-empty-vaults",
      checked: $hideZeroBalanceVaults
    },
    {},
    {}
  )}</div>`;
});
const CheckboxInactiveOrdersVault = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $hideInactiveOrdersVaults, $$unsubscribe_hideInactiveOrdersVaults;
  let { hideInactiveOrdersVaults } = $$props;
  $$unsubscribe_hideInactiveOrdersVaults = subscribe(hideInactiveOrdersVaults, (value) => $hideInactiveOrdersVaults = value);
  if ($$props.hideInactiveOrdersVaults === void 0 && $$bindings.hideInactiveOrdersVaults && hideInactiveOrdersVaults !== void 0) $$bindings.hideInactiveOrdersVaults(hideInactiveOrdersVaults);
  $$unsubscribe_hideInactiveOrdersVaults();
  return `<div data-testid="inactive-orders-vault-checkbox" class="flex items-center gap-x-2">${validate_component(Label, "Label").$$render(
    $$result,
    {
      for: "hide-inactive-orders-vaults",
      class: "cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
    },
    {},
    {
      default: () => {
        return `Hide vaults without active orders`;
      }
    }
  )} ${validate_component(Checkbox, "Checkbox").$$render(
    $$result,
    {
      id: "hide-inactive-orders-vaults",
      checked: $hideInactiveOrdersVaults
    },
    {},
    {}
  )}</div>`;
});
const DropdownActiveNetworks = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $selectedChainIds, $$unsubscribe_selectedChainIds;
  const raindexClient = useRaindexClient();
  let { selectedChainIds } = $$props;
  $$unsubscribe_selectedChainIds = subscribe(selectedChainIds, (value2) => $selectedChainIds = value2);
  let dropdownOptions = {};
  let value = {};
  if ($$props.selectedChainIds === void 0 && $$bindings.selectedChainIds && selectedChainIds !== void 0) $$bindings.selectedChainIds(selectedChainIds);
  {
    {
      const uniqueChainIds = raindexClient.getUniqueChainIds();
      if (uniqueChainIds.error) {
        dropdownOptions = {};
      } else {
        dropdownOptions = Object.fromEntries(uniqueChainIds.value.map((chainId) => [String(chainId), getNetworkName(chainId) ?? `Chain ${chainId}`]));
      }
    }
  }
  {
    {
      value = Object.fromEntries($selectedChainIds.map((chainId) => [String(chainId), getNetworkName(chainId) ?? `Chain ${chainId}`]));
    }
  }
  $$unsubscribe_selectedChainIds();
  return `<div data-testid="subgraphs-dropdown">${validate_component(DropdownCheckbox, "DropdownCheckbox").$$render(
    $$result,
    {
      options: dropdownOptions,
      label: "Networks",
      showAllLabel: false,
      onlyTitle: true,
      value
    },
    {},
    {}
  )}</div>`;
});
const CheckboxMyItemsOnly = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $showMyItemsOnly, $$unsubscribe_showMyItemsOnly;
  let $account, $$unsubscribe_account;
  let { showMyItemsOnly } = $$props;
  $$unsubscribe_showMyItemsOnly = subscribe(showMyItemsOnly, (value) => $showMyItemsOnly = value);
  let { context } = $$props;
  const { account } = useAccount();
  $$unsubscribe_account = subscribe(account, (value) => $account = value);
  if ($$props.showMyItemsOnly === void 0 && $$bindings.showMyItemsOnly && showMyItemsOnly !== void 0) $$bindings.showMyItemsOnly(showMyItemsOnly);
  if ($$props.context === void 0 && $$bindings.context && context !== void 0) $$bindings.context(context);
  $$unsubscribe_showMyItemsOnly();
  $$unsubscribe_account();
  return `<div data-testid="show-my-items-checkbox" class="flex items-center gap-x-2">${validate_component(Label, "Label").$$render(
    $$result,
    {
      for: "show-my-items",
      class: "cursor-pointer whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-300"
    },
    {},
    {
      default: () => {
        return `Only show my ${escape(context)}`;
      }
    }
  )} ${validate_component(Checkbox, "Checkbox").$$render(
    $$result,
    {
      id: "show-my-items",
      checked: $showMyItemsOnly,
      disabled: !$account
    },
    {},
    {}
  )}</div>`;
});
const ListViewOrderbookFilters = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let isVaultsPage;
  let isOrdersPage;
  let networks;
  let accounts;
  let $page, $$unsubscribe_page;
  let $account, $$unsubscribe_account;
  let $selectedChainIds, $$unsubscribe_selectedChainIds;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  let { hideZeroBalanceVaults } = $$props;
  let { hideInactiveOrdersVaults } = $$props;
  let { activeAccountsItems } = $$props;
  let { showMyItemsOnly } = $$props;
  let { selectedChainIds } = $$props;
  $$unsubscribe_selectedChainIds = subscribe(selectedChainIds, (value) => $selectedChainIds = value);
  let { showInactiveOrders } = $$props;
  let { orderHash } = $$props;
  let { activeTokens } = $$props;
  let { selectedTokens } = $$props;
  let { tokensQuery } = $$props;
  let { activeOrderbookAddresses } = $$props;
  let { selectedOrderbookAddresses } = $$props;
  const { account } = useAccount();
  $$unsubscribe_account = subscribe(account, (value) => $account = value);
  const raindexClient = useRaindexClient();
  if ($$props.hideZeroBalanceVaults === void 0 && $$bindings.hideZeroBalanceVaults && hideZeroBalanceVaults !== void 0) $$bindings.hideZeroBalanceVaults(hideZeroBalanceVaults);
  if ($$props.hideInactiveOrdersVaults === void 0 && $$bindings.hideInactiveOrdersVaults && hideInactiveOrdersVaults !== void 0) $$bindings.hideInactiveOrdersVaults(hideInactiveOrdersVaults);
  if ($$props.activeAccountsItems === void 0 && $$bindings.activeAccountsItems && activeAccountsItems !== void 0) $$bindings.activeAccountsItems(activeAccountsItems);
  if ($$props.showMyItemsOnly === void 0 && $$bindings.showMyItemsOnly && showMyItemsOnly !== void 0) $$bindings.showMyItemsOnly(showMyItemsOnly);
  if ($$props.selectedChainIds === void 0 && $$bindings.selectedChainIds && selectedChainIds !== void 0) $$bindings.selectedChainIds(selectedChainIds);
  if ($$props.showInactiveOrders === void 0 && $$bindings.showInactiveOrders && showInactiveOrders !== void 0) $$bindings.showInactiveOrders(showInactiveOrders);
  if ($$props.orderHash === void 0 && $$bindings.orderHash && orderHash !== void 0) $$bindings.orderHash(orderHash);
  if ($$props.activeTokens === void 0 && $$bindings.activeTokens && activeTokens !== void 0) $$bindings.activeTokens(activeTokens);
  if ($$props.selectedTokens === void 0 && $$bindings.selectedTokens && selectedTokens !== void 0) $$bindings.selectedTokens(selectedTokens);
  if ($$props.tokensQuery === void 0 && $$bindings.tokensQuery && tokensQuery !== void 0) $$bindings.tokensQuery(tokensQuery);
  if ($$props.activeOrderbookAddresses === void 0 && $$bindings.activeOrderbookAddresses && activeOrderbookAddresses !== void 0) $$bindings.activeOrderbookAddresses(activeOrderbookAddresses);
  if ($$props.selectedOrderbookAddresses === void 0 && $$bindings.selectedOrderbookAddresses && selectedOrderbookAddresses !== void 0) $$bindings.selectedOrderbookAddresses(selectedOrderbookAddresses);
  isVaultsPage = $page.url.pathname === "/vaults";
  isOrdersPage = $page.url.pathname === "/orders";
  networks = raindexClient.getAllNetworks();
  accounts = raindexClient.getAllAccounts();
  $$unsubscribe_page();
  $$unsubscribe_account();
  $$unsubscribe_selectedChainIds();
  return `<div class="grid w-full items-center gap-4 md:flex md:justify-end lg:min-w-[600px]" style="grid-template-columns: repeat(2, minmax(0, 1fr));">${networks.error || isEmpty(networks.value) ? `${validate_component(Alert, "Alert").$$render(
    $$result,
    {
      color: "gray",
      "data-testid": "no-networks-alert",
      class: "w-full"
    },
    {},
    {
      default: () => {
        return `No networks added to <a class="underline" href="/settings" data-svelte-h="svelte-42qdhe">settings</a>`;
      }
    }
  )}` : `${!accounts.error && accounts.value.size === 0 ? `<div class="mt-4 w-full lg:w-auto" data-testid="my-items-only">${validate_component(CheckboxMyItemsOnly, "CheckboxMyItemsOnly").$$render(
    $$result,
    {
      context: isVaultsPage ? "vaults" : "orders",
      showMyItemsOnly
    },
    {},
    {}
  )} ${!$account ? `${validate_component(Tooltip_1, "Tooltip").$$render($$result, {}, {}, {
    default: () => {
      return `Connect a wallet to filter by ${escape(isVaultsPage ? "vault" : "order")} owner`;
    }
  })}` : ``}</div>` : ``} ${isVaultsPage ? `<div class="mt-4 w-full lg:w-auto">${validate_component(CheckboxZeroBalanceVault, "CheckboxZeroBalanceVault").$$render($$result, { hideZeroBalanceVaults }, {}, {})}</div> <div class="mt-4 w-full lg:w-auto">${validate_component(CheckboxInactiveOrdersVault, "CheckboxInactiveOrdersVault").$$render($$result, { hideInactiveOrdersVaults }, {}, {})}</div>` : ``} ${isOrdersPage ? `${validate_component(InputOrderHash, "InputOrderHash").$$render($$result, { orderHash }, {}, {})} <div class="mt-4">${validate_component(CheckboxActiveOrders, "CheckboxActiveOrders").$$render($$result, { showInactiveOrders }, {}, {})}</div>` : ``} ${!accounts.error && accounts.value.size > 0 ? `${validate_component(DropdownOrderListAccounts, "DropdownOrderListAccounts").$$render($$result, { activeAccountsItems }, {}, {})}` : ``} ${validate_component(DropdownTokensFilter, "DropdownTokensFilter").$$render(
    $$result,
    {
      tokensQuery,
      activeTokens,
      selectedTokens,
      label: "Tokens"
    },
    {},
    {}
  )} ${validate_component(DropdownOrderbooksFilter, "DropdownOrderbooksFilter").$$render(
    $$result,
    {
      activeOrderbookAddresses,
      selectedOrderbookAddresses,
      selectedChainIds: $selectedChainIds,
      label: "Orderbooks"
    },
    {},
    {}
  )} ${validate_component(DropdownActiveNetworks, "DropdownActiveNetworks").$$render($$result, { selectedChainIds }, {}, {})}`}</div>`;
});
export {
  Checkbox as C,
  DotsVerticalOutline as D,
  ListViewOrderbookFilters as L,
  DropdownItem as a
};
//# sourceMappingURL=ListViewOrderbookFilters.js.map
