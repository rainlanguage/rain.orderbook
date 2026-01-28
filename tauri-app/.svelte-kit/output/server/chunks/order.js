import { c as create_ssr_component, a as compute_rest_props, b as spread, d as escape_object, e as escape_attribute_value, f as add_attribute, i as compute_slots, g as getContext, v as validate_component, h as escape, s as setContext, k as subscribe, q as get_store_value } from "./ssr.js";
import { twMerge, twJoin } from "tailwind-merge";
import { i as is_void, B as Button } from "./darkMode.js";
import { k as toasts, B as ButtonLoading } from "./sentry.js";
import { get } from "@square/svelte-store";
import { ethers } from "ethers";
import { n as walletconnectProvider, w as walletconnectAccount, l as ledgerWalletAddress, h as walletConnectNetwork, M as Modal, I as IconLedger, a as IconWalletConnect, b as InputLedgerWallet, c as InputWalletConnect, k as ledgerWalletDerivationIndex } from "./queryClient.js";
import * as chains from "viem/chains";
import { invoke } from "@tauri-apps/api";
const Wrapper = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["tag", "show", "use"]);
  let { tag = "div" } = $$props;
  let { show } = $$props;
  let { use = () => {
  } } = $$props;
  if ($$props.tag === void 0 && $$bindings.tag && tag !== void 0) $$bindings.tag(tag);
  if ($$props.show === void 0 && $$bindings.show && show !== void 0) $$bindings.show(show);
  if ($$props.use === void 0 && $$bindings.use && use !== void 0) $$bindings.use(use);
  return `${show ? `${((tag$1) => {
    return tag$1 ? `<${tag}${spread([escape_object($$restProps)], {})}>${is_void(tag$1) ? "" : `${slots.default ? slots.default({}) : ``}`}${is_void(tag$1) ? "" : `</${tag$1}>`}` : "";
  })(tag)}` : `${slots.default ? slots.default({}) : ``}`} `;
});
const Label = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let labelClass;
  let $$restProps = compute_rest_props($$props, ["color", "defaultClass", "show"]);
  let { color = "gray" } = $$props;
  let { defaultClass = "text-sm rtl:text-right font-medium block" } = $$props;
  let { show = true } = $$props;
  let node;
  const colorClasses = {
    gray: "text-gray-900 dark:text-gray-300",
    green: "text-green-700 dark:text-green-500",
    red: "text-red-700 dark:text-red-500",
    disabled: "text-gray-400 dark:text-gray-500"
  };
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.defaultClass === void 0 && $$bindings.defaultClass && defaultClass !== void 0) $$bindings.defaultClass(defaultClass);
  if ($$props.show === void 0 && $$bindings.show && show !== void 0) $$bindings.show(show);
  {
    {
      color = color;
    }
  }
  labelClass = twMerge(defaultClass, colorClasses[color], $$props.class);
  return `${show ? ` <label${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(labelClass)
      }
    ],
    {}
  )}${add_attribute("this", node, 0)}>${slots.default ? slots.default({}) : ``}</label>` : `${slots.default ? slots.default({}) : ``}`} `;
});
function clampSize(s) {
  return s && s === "xs" ? "sm" : s === "xl" ? "lg" : s;
}
const Input = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let _size;
  let $$restProps = compute_rest_props($$props, ["type", "value", "size", "defaultClass", "color", "floatClass"]);
  let $$slots = compute_slots(slots);
  let { type = "text" } = $$props;
  let { value = void 0 } = $$props;
  let { size = void 0 } = $$props;
  let { defaultClass = "block w-full disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right" } = $$props;
  let { color = "base" } = $$props;
  let { floatClass = "flex absolute inset-y-0 items-center text-gray-500 dark:text-gray-400" } = $$props;
  const borderClasses = {
    base: "border-gray-300 dark:border-gray-600",
    tinted: "border-gray-300 dark:border-gray-500",
    green: "border-green-500 dark:border-green-400",
    red: "border-red-500 dark:border-red-400"
  };
  const ringClasses = {
    base: "focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500",
    green: "focus:ring-green-500 focus:border-green-500 dark:focus:border-green-500 dark:focus:ring-green-500",
    red: "focus:ring-red-500 focus:border-red-500 dark:focus:ring-red-500 dark:focus:border-red-500"
  };
  const colorClasses = {
    base: "bg-gray-50 text-gray-900 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400",
    tinted: "bg-gray-50 text-gray-900 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400",
    green: "bg-green-50 text-green-900 placeholder-green-700 dark:text-green-400 dark:placeholder-green-500 dark:bg-gray-700",
    red: "bg-red-50 text-red-900 placeholder-red-700 dark:text-red-500 dark:placeholder-red-500 dark:bg-gray-700"
  };
  let background = getContext("background");
  let group = getContext("group");
  const textSizes = {
    sm: "sm:text-xs",
    md: "text-sm",
    lg: "sm:text-base"
  };
  const leftPadding = { sm: "ps-9", md: "ps-10", lg: "ps-11" };
  const rightPadding = { sm: "pe-9", md: "pe-10", lg: "pe-11" };
  const inputPadding = { sm: "p-2", md: "p-2.5", lg: "p-3" };
  let inputClass;
  if ($$props.type === void 0 && $$bindings.type && type !== void 0) $$bindings.type(type);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.defaultClass === void 0 && $$bindings.defaultClass && defaultClass !== void 0) $$bindings.defaultClass(defaultClass);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.floatClass === void 0 && $$bindings.floatClass && floatClass !== void 0) $$bindings.floatClass(floatClass);
  _size = size || clampSize(group?.size) || "md";
  {
    {
      const _color = color === "base" && background ? "tinted" : color;
      inputClass = twMerge([
        defaultClass,
        inputPadding[_size],
        $$slots.left && leftPadding[_size] || $$slots.right && rightPadding[_size],
        ringClasses[color],
        colorClasses[_color],
        borderClasses[_color],
        textSizes[_size],
        group || "rounded-lg",
        group && "first:rounded-s-lg last:rounded-e-lg",
        group && "border-s-0 first:border-s last:border-e",
        $$props.class
      ]);
    }
  }
  return `${validate_component(Wrapper, "Wrapper").$$render(
    $$result,
    {
      class: "relative w-full",
      show: $$slots.left || $$slots.right
    },
    {},
    {
      default: () => {
        return `${$$slots.left ? `<div class="${escape(twMerge(floatClass, $$props.classLeft), true) + " start-0 ps-2.5 pointer-events-none"}">${slots.left ? slots.left({}) : ``}</div>` : ``} ${slots.default ? slots.default({
          props: { ...$$restProps, class: inputClass }
        }) : ` <input${spread(
          [
            escape_object($$restProps),
            escape_object({ type }),
            {
              class: escape_attribute_value(inputClass)
            }
          ],
          {}
        )}${add_attribute("value", value, 0)}> `} ${$$slots.right ? `<div class="${escape(twMerge(floatClass, $$props.classRight), true) + " end-0 pe-2.5"}">${slots.right ? slots.right({}) : ``}</div>` : ``}`;
      }
    }
  )} `;
});
const Table = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["divClass", "striped", "hoverable", "noborder", "shadow", "color", "customeColor"]);
  let { divClass = "relative overflow-x-auto" } = $$props;
  let { striped = false } = $$props;
  let { hoverable = false } = $$props;
  let { noborder = false } = $$props;
  let { shadow = false } = $$props;
  let { color = "default" } = $$props;
  let { customeColor = "" } = $$props;
  const colors = {
    default: "text-gray-500 dark:text-gray-400",
    blue: "text-blue-100 dark:text-blue-100",
    green: "text-green-100 dark:text-green-100",
    red: "text-red-100 dark:text-red-100",
    yellow: "text-yellow-100 dark:text-yellow-100",
    purple: "text-purple-100 dark:text-purple-100",
    indigo: "text-indigo-100 dark:text-indigo-100",
    pink: "text-pink-100 dark:text-pink-100",
    custom: customeColor
  };
  if ($$props.divClass === void 0 && $$bindings.divClass && divClass !== void 0) $$bindings.divClass(divClass);
  if ($$props.striped === void 0 && $$bindings.striped && striped !== void 0) $$bindings.striped(striped);
  if ($$props.hoverable === void 0 && $$bindings.hoverable && hoverable !== void 0) $$bindings.hoverable(hoverable);
  if ($$props.noborder === void 0 && $$bindings.noborder && noborder !== void 0) $$bindings.noborder(noborder);
  if ($$props.shadow === void 0 && $$bindings.shadow && shadow !== void 0) $$bindings.shadow(shadow);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.customeColor === void 0 && $$bindings.customeColor && customeColor !== void 0) $$bindings.customeColor(customeColor);
  {
    setContext("striped", striped);
  }
  {
    setContext("hoverable", hoverable);
  }
  {
    setContext("noborder", noborder);
  }
  {
    setContext("color", color);
  }
  return `<div${add_attribute("class", twJoin(divClass, shadow && "shadow-md sm:rounded-lg"), 0)}><table${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge("w-full text-left text-sm", colors[color], $$props.class))
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</table></div> `;
});
const TableBody = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { tableBodyClass = void 0 } = $$props;
  if ($$props.tableBodyClass === void 0 && $$bindings.tableBodyClass && tableBodyClass !== void 0) $$bindings.tableBodyClass(tableBodyClass);
  return `<tbody${add_attribute("class", tableBodyClass, 0)}>${slots.default ? slots.default({}) : ``}</tbody> `;
});
const TableBodyCell = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["tdClass"]);
  let { tdClass = "px-6 py-4 whitespace-nowrap font-medium " } = $$props;
  let color = "default";
  color = getContext("color");
  let tdClassfinal;
  if ($$props.tdClass === void 0 && $$bindings.tdClass && tdClass !== void 0) $$bindings.tdClass(tdClass);
  tdClassfinal = twMerge(
    tdClass,
    color === "default" ? "text-gray-900 dark:text-white" : "text-blue-50 whitespace-nowrap dark:text-blue-100",
    $$props.class
  );
  return `${((tag) => {
    return tag ? `<${$$props.onclick ? "button" : "td"}${spread(
      [
        escape_object($$restProps),
        {
          class: escape_attribute_value(tdClassfinal)
        },
        {
          role: escape_attribute_value($$props.onclick ? "button" : void 0)
        }
      ],
      {}
    )}>${is_void(tag) ? "" : `${slots.default ? slots.default({}) : ``}`}${is_void(tag) ? "" : `</${tag}>`}` : "";
  })($$props.onclick ? "button" : "td")} `;
});
const TableBodyRow = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["color"]);
  let { color = getContext("color") } = $$props;
  const colors = {
    default: "bg-white dark:bg-gray-800 dark:border-gray-700",
    blue: "bg-blue-500 border-blue-400",
    green: "bg-green-500 border-green-400",
    red: "bg-red-500 border-red-400",
    yellow: "bg-yellow-500 border-yellow-400",
    purple: "bg-purple-500 border-purple-400",
    custom: ""
  };
  const hoverColors = {
    default: "hover:bg-gray-50 dark:hover:bg-gray-600",
    blue: "hover:bg-blue-400",
    green: "hover:bg-green-400",
    red: "hover:bg-red-400",
    yellow: "hover:bg-yellow-400",
    purple: "hover:bg-purple-400",
    custom: ""
  };
  const stripColors = {
    default: "odd:bg-white even:bg-gray-50 odd:dark:bg-gray-800 even:dark:bg-gray-700",
    blue: "odd:bg-blue-800 even:bg-blue-700 odd:dark:bg-blue-800 even:dark:bg-blue-700",
    green: "odd:bg-green-800 even:bg-green-700 odd:dark:bg-green-800 even:dark:bg-green-700",
    red: "odd:bg-red-800 even:bg-red-700 odd:dark:bg-red-800 even:dark:bg-red-700",
    yellow: "odd:bg-yellow-800 even:bg-yellow-700 odd:dark:bg-yellow-800 even:dark:bg-yellow-700",
    purple: "odd:bg-purple-800 even:bg-purple-700 odd:dark:bg-purple-800 even:dark:bg-purple-700",
    custom: ""
  };
  let trClass;
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  trClass = twMerge([
    !getContext("noborder") && "border-b last:border-b-0",
    colors[color],
    getContext("hoverable") && hoverColors[color],
    getContext("striped") && stripColors[color],
    $$props.class
  ]);
  return `<tr${spread([escape_object($$restProps), { class: escape_attribute_value(trClass) }], {})}>${slots.default ? slots.default({}) : ``}</tr> `;
});
const TableHead = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let theadClassfinal;
  let $$restProps = compute_rest_props($$props, ["theadClass", "defaultRow"]);
  let { theadClass = "text-xs uppercase" } = $$props;
  let { defaultRow = true } = $$props;
  let color;
  color = getContext("color");
  let noborder = getContext("noborder");
  let striped = getContext("striped");
  let defaultBgColor = noborder || striped ? "" : "bg-gray-50 dark:bg-gray-700";
  const bgColors = {
    default: defaultBgColor,
    blue: "bg-blue-600",
    green: "bg-green-600",
    red: "bg-red-600",
    yellow: "bg-yellow-600",
    purple: "bg-purple-600",
    custom: ""
  };
  let textColor = color === "default" ? "text-gray-700 dark:text-gray-400" : color === "custom" ? "" : "text-white  dark:text-white";
  let borderColors = striped ? "" : color === "default" ? "border-gray-700" : color === "custom" ? "" : `border-${color}-400`;
  if ($$props.theadClass === void 0 && $$bindings.theadClass && theadClass !== void 0) $$bindings.theadClass(theadClass);
  if ($$props.defaultRow === void 0 && $$bindings.defaultRow && defaultRow !== void 0) $$bindings.defaultRow(defaultRow);
  theadClassfinal = twMerge(theadClass, textColor, striped && borderColors, bgColors[color], $$props.class);
  return `<thead${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(theadClassfinal)
      }
    ],
    {}
  )}>${defaultRow ? `<tr>${slots.default ? slots.default({}) : ``}</tr>` : `${slots.default ? slots.default({}) : ``}`}</thead> `;
});
const TableHeadCell = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["padding"]);
  let { padding = "px-6 py-3" } = $$props;
  if ($$props.padding === void 0 && $$bindings.padding && padding !== void 0) $$bindings.padding(padding);
  return `<th${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(padding, $$props.class))
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</th> `;
});
const Refresh = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "role", "ariaLabel", "spin", "testId"]);
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
  let { ariaLabel = "refresh" } = $$props;
  let { spin = false } = $$props;
  let { testId = "refresh-button" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  if ($$props.spin === void 0 && $$bindings.spin && spin !== void 0) $$bindings.spin(spin);
  if ($$props.testId === void 0 && $$bindings.testId && testId !== void 0) $$bindings.testId(testId);
  return `<svg${spread(
    [
      {
        "data-testid": escape_attribute_value(testId)
      },
      { xmlns: "http://www.w3.org/2000/svg" },
      { fill: "none" },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge("shrink-0 cursor-pointer outline-none", sizes[size], $$props.class, spin ? "animate-spin ease-out" : ""))
      },
      { role: escape_attribute_value(role) },
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      { viewBox: "0 0 24 24" }
    ],
    {}
  )}><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.651 7.65a7.131 7.131 0 0 0-12.68 3.15M18.001 4v4h-4m-7.652 8.35a7.13 7.13 0 0 0 12.68-3.15M6 20v-4h4"></path></svg> `;
});
function getNetworkName(chainId) {
  const chain = Object.values(chains).find((chain2) => chain2.id === chainId);
  return chain?.name;
}
async function ethersExecute(calldata, to) {
  if (!walletconnectProvider || !get(walletconnectAccount)) {
    toasts.error("user not connected");
    return Promise.reject("user not connected");
  } else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    const rawtx = {
      data: calldata,
      to
    };
    return signer.sendTransaction(rawtx);
  }
}
const ModalExecute = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $walletconnectAccount, $$unsubscribe_walletconnectAccount;
  let $ledgerWalletAddress, $$unsubscribe_ledgerWalletAddress;
  let $walletConnectNetwork, $$unsubscribe_walletConnectNetwork;
  $$unsubscribe_walletconnectAccount = subscribe(walletconnectAccount, (value) => $walletconnectAccount = value);
  $$unsubscribe_ledgerWalletAddress = subscribe(ledgerWalletAddress, (value) => $ledgerWalletAddress = value);
  $$unsubscribe_walletConnectNetwork = subscribe(walletConnectNetwork, (value) => $walletConnectNetwork = value);
  let { open = false } = $$props;
  let { title } = $$props;
  let { execButtonLabel } = $$props;
  let { executeLedger } = $$props;
  let { executeWalletconnect } = $$props;
  let { isSubmitting = false } = $$props;
  let { onBack = void 0 } = $$props;
  let { chainId = void 0 } = $$props;
  let { overrideNetwork = void 0 } = $$props;
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.title === void 0 && $$bindings.title && title !== void 0) $$bindings.title(title);
  if ($$props.execButtonLabel === void 0 && $$bindings.execButtonLabel && execButtonLabel !== void 0) $$bindings.execButtonLabel(execButtonLabel);
  if ($$props.executeLedger === void 0 && $$bindings.executeLedger && executeLedger !== void 0) $$bindings.executeLedger(executeLedger);
  if ($$props.executeWalletconnect === void 0 && $$bindings.executeWalletconnect && executeWalletconnect !== void 0) $$bindings.executeWalletconnect(executeWalletconnect);
  if ($$props.isSubmitting === void 0 && $$bindings.isSubmitting && isSubmitting !== void 0) $$bindings.isSubmitting(isSubmitting);
  if ($$props.onBack === void 0 && $$bindings.onBack && onBack !== void 0) $$bindings.onBack(onBack);
  if ($$props.chainId === void 0 && $$bindings.chainId && chainId !== void 0) $$bindings.chainId(chainId);
  if ($$props.overrideNetwork === void 0 && $$bindings.overrideNetwork && overrideNetwork !== void 0) $$bindings.overrideNetwork(overrideNetwork);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title,
        outsideclose: !isSubmitting,
        size: "sm",
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
          return `${!$walletconnectAccount && !$ledgerWalletAddress ? `<div class="flex justify-center space-x-4">${validate_component(Button, "Button").$$render($$result, { class: "text-lg" }, {}, {
            default: () => {
              return `<div class="mr-4">${validate_component(IconLedger, "IconLedger").$$render($$result, {}, {}, {})}</div>
        Ledger Wallet`;
            }
          })} ${validate_component(Button, "Button").$$render($$result, { class: "text-lg" }, {}, {
            default: () => {
              return `<div class="mr-3">${validate_component(IconWalletConnect, "IconWalletConnect").$$render($$result, {}, {}, {})}</div>
        WalletConnect`;
            }
          })}</div> <div class="flex justify-end space-x-4">${onBack ? `${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Back`;
            }
          })}` : ``}</div>` : `${$ledgerWalletAddress ? `${validate_component(InputLedgerWallet, "InputLedgerWallet").$$render($$result, {}, {}, {})} <div${add_attribute(
            "class",
            !$ledgerWalletAddress ? "flex justify-between space-x-4" : "flex justify-end space-x-4",
            0
          )}>${!$ledgerWalletAddress ? `${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Back`;
            }
          })}` : ``} ${validate_component(ButtonLoading, "ButtonLoading").$$render(
            $$result,
            {
              disabled: isSubmitting || !$ledgerWalletAddress,
              loading: isSubmitting
            },
            {},
            {
              default: () => {
                return `${escape(execButtonLabel)}`;
              }
            }
          )}</div>` : `${$walletconnectAccount ? `${validate_component(InputWalletConnect, "InputWalletConnect").$$render(
            $$result,
            {
              priorityChainIds: chainId ? [chainId] : []
            },
            {},
            {}
          )} <div${add_attribute(
            "class",
            !$walletconnectAccount ? "flex items-center justify-between space-x-4" : "flex items-center justify-end space-x-4",
            0
          )}>${!$walletconnectAccount ? `${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Back`;
            }
          })}` : ``} ${validate_component(ButtonLoading, "ButtonLoading").$$render(
            $$result,
            {
              disabled: isSubmitting || !$walletconnectAccount || $walletConnectNetwork !== chainId,
              loading: isSubmitting
            },
            {},
            {
              default: () => {
                return `${escape(execButtonLabel)}`;
              }
            }
          )} ${$walletconnectAccount && $walletConnectNetwork !== chainId ? `<div class="text-red-500" data-testid="network-connection-error">You are connected to ${escape(getNetworkName($walletConnectNetwork) || "an unknown")} network. Please
          connect your wallet to ${escape(overrideNetwork?.key || getNetworkName(chainId ?? 0) || "unknown")}
          network.</div>` : ``}</div>` : ``}`}`}`;
        }
      }
    )}`;
  } while (!$$settled);
  $$unsubscribe_walletconnectAccount();
  $$unsubscribe_ledgerWalletAddress();
  $$unsubscribe_walletConnectNetwork();
  return $$rendered;
});
async function orderAdd(dotrain, deployment) {
  await invoke("order_add", {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: get_store_value(ledgerWalletDerivationIndex),
      chain_id: deployment.order.network.chainId
    }
  });
}
async function orderRemove(raindexClient, order) {
  const orderbook = raindexClient.getOrderbookByAddress(order.orderbook);
  if (orderbook.error) {
    throw new Error(orderbook.error.readableMsg);
  }
  await invoke("order_remove", {
    order,
    transactionArgs: {
      rpcs: orderbook.value.network.rpcs,
      orderbook_address: order.orderbook,
      derivation_index: get_store_value(ledgerWalletDerivationIndex),
      chain_id: order.chainId
    },
    subgraphArgs: {
      url: orderbook.value.subgraph.url
    }
  });
}
async function orderAddCalldata(dotrain, deployment) {
  return await invoke("order_add_calldata", {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: void 0,
      chain_id: deployment.order.network.chainId
    }
  });
}
async function orderAddComposeRainlang(dotrain, settings, scenario) {
  return await invoke("compose_from_scenario", {
    dotrain,
    settings,
    scenario
  });
}
async function validateSpecVersion(dotrain, settings) {
  return await invoke("validate_spec_version", {
    dotrain,
    settings
  });
}
export {
  Input as I,
  Label as L,
  ModalExecute as M,
  Refresh as R,
  TableBodyCell as T,
  Wrapper as W,
  TableHeadCell as a,
  Table as b,
  TableHead as c,
  TableBody as d,
  TableBodyRow as e,
  orderAdd as f,
  getNetworkName as g,
  orderAddCalldata as h,
  ethersExecute as i,
  clampSize as j,
  orderRemove as k,
  orderAddComposeRainlang as o,
  validateSpecVersion as v
};
//# sourceMappingURL=order.js.map
