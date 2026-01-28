import { c as create_ssr_component, a as compute_rest_props, b as spread, e as escape_attribute_value, d as escape_object, f as add_attribute, s as setContext, g as getContext, h as escape, i as compute_slots, v as validate_component, j as each, k as subscribe } from "../../chunks/ssr.js";
import { twMerge } from "tailwind-merge";
import { w as writable, r as readable, d as derived } from "../../chunks/index.js";
import { p as page } from "../../chunks/stores.js";
import { B as Button, c as colorTheme } from "../../chunks/darkMode.js";
import { R as RAINDEX_CLIENT_CONTEXT_KEY, l as ledgerWalletAddress, w as walletconnectAccount, M as Modal, I as IconLedger, a as IconWalletConnect, b as InputLedgerWallet, c as InputWalletConnect, d as IconWarning, E as ExclamationCircleSolid, f as formatBlockExplorerTransactionUrl, q as queryClient } from "../../chunks/queryClient.js";
import "@tauri-apps/api";
import { l as logoDark, a as logoLight, I as IconTelegram } from "../../chunks/logo-dark.js";
import { T as TransitionFrame, F as Frame, C as CloseButton, s as slide, a as ToastMessageType, S as Spinner, b as settingsText, t as toastsList, R as RaindexClient, v as validChainIds } from "../../chunks/sentry.js";
import { listen } from "@tauri-apps/api/event";
import sortBy from "lodash/sortBy.js";
import { platform } from "@tauri-apps/api/os";
import { QueryClientProvider } from "@tanstack/svelte-query";
import "@sentry/sveltekit";
import "@tauri-apps/api/app";
import { u as useToasts, s as setToastsContext } from "../../chunks/useToasts.js";
import { s as setAccountContext } from "../../chunks/context.js";
const DarkMode = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["btnClass", "size", "ariaLabel"]);
  let { btnClass = "text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none rounded-lg text-sm p-2.5" } = $$props;
  let { size = "md" } = $$props;
  let { ariaLabel = "Dark mode" } = $$props;
  const sizes = {
    sm: "w-4 h-4",
    md: "w-5 h-5",
    lg: "w-6 h-6"
  };
  if ($$props.btnClass === void 0 && $$bindings.btnClass && btnClass !== void 0) $$bindings.btnClass(btnClass);
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  return `${$$result.head += `<!-- HEAD_svelte-1pa505f_START --><script data-svelte-h="svelte-mp99qu">if ('color-theme' in localStorage) {
      // explicit preference - overrides author's choice
      localStorage.getItem('color-theme') === 'dark' ? window.document.documentElement.classList.add('dark') : window.document.documentElement.classList.remove('dark');
    } else {
      // browser preference - does not overrides
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) window.document.documentElement.classList.add('dark');
    }<\/script><!-- HEAD_svelte-1pa505f_END -->`, ""} <button${spread(
    [
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      { type: "button" },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(btnClass, $$props.class))
      }
    ],
    {}
  )}><span class="hidden dark:block">${slots.lightIcon ? slots.lightIcon({}) : ` <svg${add_attribute("class", sizes[size], 0)} fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1
  0 100-2H3a1 1 0 000 2h1z" fill-rule="evenodd" clip-rule="evenodd"></path></svg> `}</span> <span class="block dark:hidden">${slots.darkIcon ? slots.darkIcon({}) : ` <svg${add_attribute("class", sizes[size], 0)} fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path></svg> `}</span></button> `;
});
const Sidebar = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["activeUrl", "asideClass", "nonActiveClass", "activeClass", "ariaLabel"]);
  const activeUrlStore = writable("");
  let { activeUrl = "" } = $$props;
  let { asideClass = "w-64" } = $$props;
  let { nonActiveClass = "flex items-center p-2 text-base font-normal text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700" } = $$props;
  let { activeClass = "flex items-center p-2 text-base font-normal text-gray-900 bg-gray-200 dark:bg-gray-700 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700" } = $$props;
  let { ariaLabel = "Sidebar" } = $$props;
  setContext("sidebarContext", { activeClass, nonActiveClass });
  setContext("activeUrl", activeUrlStore);
  if ($$props.activeUrl === void 0 && $$bindings.activeUrl && activeUrl !== void 0) $$bindings.activeUrl(activeUrl);
  if ($$props.asideClass === void 0 && $$bindings.asideClass && asideClass !== void 0) $$bindings.asideClass(asideClass);
  if ($$props.nonActiveClass === void 0 && $$bindings.nonActiveClass && nonActiveClass !== void 0) $$bindings.nonActiveClass(nonActiveClass);
  if ($$props.activeClass === void 0 && $$bindings.activeClass && activeClass !== void 0) $$bindings.activeClass(activeClass);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  {
    {
      activeUrlStore.set(activeUrl);
    }
  }
  return `<aside${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(asideClass, $$props.class))
      },
      {
        "aria-label": escape_attribute_value(ariaLabel)
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</aside> `;
});
const SidebarItem = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let active;
  let aClass;
  let $$restProps = compute_rest_props($$props, ["href", "label", "spanClass", "activeClass", "nonActiveClass"]);
  let $$slots = compute_slots(slots);
  let { href = "" } = $$props;
  let { label = "" } = $$props;
  let { spanClass = "ms-3" } = $$props;
  let { activeClass = void 0 } = $$props;
  let { nonActiveClass = void 0 } = $$props;
  const context = getContext("sidebarContext") ?? {};
  const activeUrlStore = getContext("activeUrl");
  let sidebarUrl = "";
  activeUrlStore.subscribe((value) => {
    sidebarUrl = value;
  });
  if ($$props.href === void 0 && $$bindings.href && href !== void 0) $$bindings.href(href);
  if ($$props.label === void 0 && $$bindings.label && label !== void 0) $$bindings.label(label);
  if ($$props.spanClass === void 0 && $$bindings.spanClass && spanClass !== void 0) $$bindings.spanClass(spanClass);
  if ($$props.activeClass === void 0 && $$bindings.activeClass && activeClass !== void 0) $$bindings.activeClass(activeClass);
  if ($$props.nonActiveClass === void 0 && $$bindings.nonActiveClass && nonActiveClass !== void 0) $$bindings.nonActiveClass(nonActiveClass);
  active = sidebarUrl ? href === sidebarUrl : false;
  aClass = twMerge(
    active ? activeClass ?? context.activeClass : nonActiveClass ?? context.nonActiveClass,
    $$props.class
  );
  return `<li><a${spread(
    [
      escape_object($$restProps),
      { href: escape_attribute_value(href) },
      { class: escape_attribute_value(aClass) }
    ],
    {}
  )}>${slots.icon ? slots.icon({}) : ``} <span${add_attribute("class", spanClass, 0)}>${escape(label)}</span> ${$$slots.subtext ? `${slots.subtext ? slots.subtext({}) : ``}` : ``}</a></li> `;
});
const SidebarBrand = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["site", "aClass", "imgClass", "spanClass"]);
  let { site } = $$props;
  let { aClass = "flex items-center ps-2.5 mb-5" } = $$props;
  let { imgClass = "h-6 me-3 sm:h-7" } = $$props;
  let { spanClass = "self-center text-xl font-semibold whitespace-nowrap dark:text-white" } = $$props;
  if ($$props.site === void 0 && $$bindings.site && site !== void 0) $$bindings.site(site);
  if ($$props.aClass === void 0 && $$bindings.aClass && aClass !== void 0) $$bindings.aClass(aClass);
  if ($$props.imgClass === void 0 && $$bindings.imgClass && imgClass !== void 0) $$bindings.imgClass(imgClass);
  if ($$props.spanClass === void 0 && $$bindings.spanClass && spanClass !== void 0) $$bindings.spanClass(spanClass);
  return `<a${spread(
    [
      escape_object($$restProps),
      { href: escape_attribute_value(site.href) },
      {
        class: escape_attribute_value(twMerge(aClass, $$props.class))
      }
    ],
    {}
  )}><img${add_attribute("src", site.img, 0)}${add_attribute("class", imgClass, 0)}${add_attribute("alt", site.name, 0)}> <span${add_attribute("class", spanClass, 0)}>${escape(site.name)}</span></a> `;
});
const SidebarGroup = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["ulClass", "borderClass", "border"]);
  let { ulClass = "space-y-2" } = $$props;
  let { borderClass = "pt-4 mt-4 border-t border-gray-200 dark:border-gray-700" } = $$props;
  let { border = false } = $$props;
  if (border) {
    ulClass += " " + borderClass;
  }
  if ($$props.ulClass === void 0 && $$bindings.ulClass && ulClass !== void 0) $$bindings.ulClass(ulClass);
  if ($$props.borderClass === void 0 && $$bindings.borderClass && borderClass !== void 0) $$bindings.borderClass(borderClass);
  if ($$props.border === void 0 && $$bindings.border && border !== void 0) $$bindings.border(border);
  return `<ul${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(ulClass, $$props.class))
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</ul> `;
});
const SidebarWrapper = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["divClass"]);
  let { divClass = "overflow-y-auto py-4 px-3 bg-gray-50 rounded dark:bg-gray-800" } = $$props;
  if ($$props.divClass === void 0 && $$bindings.divClass && divClass !== void 0) $$bindings.divClass(divClass);
  return `<div${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(divClass, $$props.class))
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</div> `;
});
const clsBtnExtraClass = "-mx-1.5 -my-1.5 text-gray-400 hover:text-gray-900 focus:!ring-gray-300 hover:bg-gray-100 dark:text-gray-500 dark:hover:text-white dark:hover:bg-gray-700";
const Toast = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, [
    "dismissable",
    "color",
    "position",
    "divClass",
    "defaultIconClass",
    "contentClass",
    "align"
  ]);
  let $$slots = compute_slots(slots);
  let { dismissable = true } = $$props;
  let { color = "primary" } = $$props;
  let { position = "none" } = $$props;
  let { divClass = "w-full max-w-xs p-4 text-gray-500 bg-white shadow dark:text-gray-400 dark:bg-gray-800 gap-3" } = $$props;
  let { defaultIconClass = "w-8 h-8" } = $$props;
  let { contentClass = "w-full text-sm font-normal" } = $$props;
  let { align = true } = $$props;
  const positions = {
    "top-left": "absolute top-5 start-5",
    "top-right": "absolute top-5 end-5",
    "bottom-left": "absolute bottom-5 start-5",
    "bottom-right": "absolute bottom-5 end-5",
    none: ""
  };
  let finalDivClass;
  const colors = {
    primary: "text-primary-500 bg-primary-100 dark:bg-primary-800 dark:text-primary-200",
    gray: "text-gray-500 bg-gray-100 dark:bg-gray-700 dark:text-gray-200",
    red: "text-red-500 bg-red-100 dark:bg-red-800 dark:text-red-200",
    yellow: "text-yellow-500 bg-yellow-100 dark:bg-yellow-800 dark:text-yellow-200",
    green: "text-green-500 bg-green-100 dark:bg-green-800 dark:text-green-200",
    blue: "text-blue-500 bg-blue-100 dark:bg-blue-800 dark:text-blue-200",
    indigo: "text-indigo-500 bg-indigo-100 dark:bg-indigo-800 dark:text-indigo-200",
    purple: "text-purple-500 bg-purple-100 dark:bg-purple-800 dark:text-purple-200",
    orange: "text-orange-500 bg-orange-100 dark:bg-orange-700 dark:text-orange-200",
    none: ""
  };
  let iconClass;
  if ($$props.dismissable === void 0 && $$bindings.dismissable && dismissable !== void 0) $$bindings.dismissable(dismissable);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.position === void 0 && $$bindings.position && position !== void 0) $$bindings.position(position);
  if ($$props.divClass === void 0 && $$bindings.divClass && divClass !== void 0) $$bindings.divClass(divClass);
  if ($$props.defaultIconClass === void 0 && $$bindings.defaultIconClass && defaultIconClass !== void 0) $$bindings.defaultIconClass(defaultIconClass);
  if ($$props.contentClass === void 0 && $$bindings.contentClass && contentClass !== void 0) $$bindings.contentClass(contentClass);
  if ($$props.align === void 0 && $$bindings.align && align !== void 0) $$bindings.align(align);
  finalDivClass = twMerge("flex", align ? "items-center" : "items-start", divClass, positions[position], $$props.class);
  iconClass = twMerge("inline-flex items-center justify-center shrink-0", colors[color], defaultIconClass);
  return `${validate_component(TransitionFrame, "TransitionFrame").$$render($$result, Object.assign({}, { rounded: true }, { dismissable }, { color: "none" }, { role: "alert" }, $$restProps, { class: finalDivClass }), {}, {
    default: ({ close }) => {
      return `${$$slots.icon ? `${validate_component(Frame, "Frame").$$render(
        $$result,
        {
          rounded: true,
          color: "none",
          class: iconClass
        },
        {},
        {
          default: () => {
            return `${slots.icon ? slots.icon({}) : ``}`;
          }
        }
      )}` : ``} <div${add_attribute("class", contentClass, 0)}>${slots.default ? slots.default({}) : ``}</div> ${dismissable ? `${slots["close-button"] ? slots["close-button"]({ close }) : ` ${validate_component(CloseButton, "CloseButton").$$render($$result, { class: clsBtnExtraClass }, {}, {})} `}` : ``}`;
    }
  })} `;
});
const WalletSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "wallet solid" } = $$props;
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
  )}><g fill="currentColor"><path d="M11.074 4 8.442.408A.95.95 0 0 0 7.014.254L2.926 4h8.148ZM9 13v-1a4 4 0 0 1 4-4h6V6a1 1 0 0 0-1-1H1a1 1 0 0 0-1 1v13a1 1 0 0 0 1 1h17a1 1 0 0 0 1-1v-2h-6a4 4 0 0 1-4-4Z"></path><path d="M19 10h-6a2 2 0 0 0-2 2v1a2 2 0 0 0 2 2h6a1 1 0 0 0 1-1v-3a1 1 0 0 0-1-1Zm-4.5 3.5a1 1 0 1 1 0-2.002 1 1 0 0 1 0 2.002ZM12.62 4h2.78L12.539.409a1.086 1.086 0 1 0-1.7 1.353L12.62 4Z"></path></g></svg> `;
});
const ReceiptSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "receipt solid" } = $$props;
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
      { viewBox: "0 0 14 20" }
    ],
    {}
  )}><path fill="currentColor" d="M13.383.076a1 1 0 0 0-1.09.217L11 1.586 9.707.293a1 1 0 0 0-1.414 0L7 1.586 5.707.293a1 1 0 0 0-1.414 0L3 1.586 1.707.293A1 1 0 0 0 0 1v18a1 1 0 0 0 1.707.707L3 18.414l1.293 1.293a1 1 0 0 0 1.414 0L7 18.414l1.293 1.293a1 1 0 0 0 1.414 0L11 18.414l1.293 1.293A1 1 0 0 0 14 19V1a1 1 0 0 0-.617-.924ZM10 15H4a1 1 0 0 1 0-2h6a1 1 0 0 1 0 2Zm0-4H4a1 1 0 0 1 0-2h6a1 1 0 1 1 0 2Zm0-4H4a1 1 0 1 1 0-2h6a1 1 0 1 1 0 2Z"></path></svg> `;
});
const GearSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "gear solid" } = $$props;
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
  )}><path fill="currentColor" d="M18 7.5h-.423l-.452-1.09.3-.3a1.5 1.5 0 0 0 0-2.121L16.01 2.575a1.5 1.5 0 0 0-2.121 0l-.3.3-1.089-.452V2A1.5 1.5 0 0 0 11 .5H9A1.5 1.5 0 0 0 7.5 2v.423l-1.09.452-.3-.3a1.5 1.5 0 0 0-2.121 0L2.576 3.99a1.5 1.5 0 0 0 0 2.121l.3.3L2.423 7.5H2A1.5 1.5 0 0 0 .5 9v2A1.5 1.5 0 0 0 2 12.5h.423l.452 1.09-.3.3a1.5 1.5 0 0 0 0 2.121l1.415 1.413a1.5 1.5 0 0 0 2.121 0l.3-.3 1.09.452V18A1.5 1.5 0 0 0 9 19.5h2a1.5 1.5 0 0 0 1.5-1.5v-.423l1.09-.452.3.3a1.5 1.5 0 0 0 2.121 0l1.415-1.414a1.5 1.5 0 0 0 0-2.121l-.3-.3.452-1.09H18a1.5 1.5 0 0 0 1.5-1.5V9A1.5 1.5 0 0 0 18 7.5Zm-8 6a3.5 3.5 0 1 1 0-7 3.5 3.5 0 0 1 0 7Z"></path></svg> `;
});
const FileLinesSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "file lines solid" } = $$props;
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
      { viewBox: "0 0 16 20" }
    ],
    {}
  )}><g fill="currentColor"><path d="M14.066 0H7v5a2 2 0 0 1-2 2H0v11a1.97 1.97 0 0 0 1.934 2h12.132A1.97 1.97 0 0 0 16 18V2a1.97 1.97 0 0 0-1.934-2Zm-3 15H4.828a1 1 0 0 1 0-2h6.238a1 1 0 0 1 0 2Zm0-4H4.828a1 1 0 0 1 0-2h6.238a1 1 0 1 1 0 2Z"></path><path d="M5 5V.13a2.96 2.96 0 0 0-1.293.749L.879 3.707A2.98 2.98 0 0 0 .13 5H5Z"></path></g></svg> `;
});
const CheckCircleSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "check circle solid" } = $$props;
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
  )}><path fill="currentColor" d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z"></path></svg> `;
});
const CloseCircleSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "close circle solid" } = $$props;
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
  )}><path fill="currentColor" d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"></path></svg> `;
});
const CloseSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "close solid" } = $$props;
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
      { viewBox: "0 0 16 16" }
    ],
    {}
  )}><path fill="currentColor" d="m9.414 8 5.293-5.293a1 1 0 1 0-1.414-1.414L8 6.586 2.707 1.293a1 1 0 0 0-1.414 1.414L6.586 8l-5.293 5.293a1 1 0 1 0 1.414 1.414L8 9.414l5.293 5.293a1 1 0 0 0 1.414-1.414L9.414 8Z"></path></svg> `;
});
const InfoCircleSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "info circle solid" } = $$props;
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
  )}><path fill="currentColor" d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5ZM9.5 4a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3ZM12 15H8a1 1 0 0 1 0-2h1v-3H8a1 1 0 0 1 0-2h2a1 1 0 0 1 1 1v4h1a1 1 0 0 1 0 2Z"></path></svg> `;
});
const PlusSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "plus solid" } = $$props;
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
      { viewBox: "0 0 18 18" }
    ],
    {}
  )}><path fill="currentColor" d="M17 8h-7V1a1 1 0 0 0-2 0v7H1a1 1 0 0 0 0 2h7v7a1 1 0 1 0 2 0v-7h7a1 1 0 1 0 0-2Z"></path></svg> `;
});
const IconError = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  return `<div class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-red-100 text-red-500 dark:bg-red-800 dark:text-red-200">${validate_component(CloseCircleSolid, "CloseCircleSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})}</div>`;
});
const IconExternalLink = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { svgClass = "" } = $$props;
  if ($$props.svgClass === void 0 && $$bindings.svgClass && svgClass !== void 0) $$bindings.svgClass(svgClass);
  return `<svg class="${"h-5 w-5 text-gray-800 dark:text-white " + escape(svgClass, true)}" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 18 18"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11v4.833A1.166 1.166 0 0 1 13.833 17H2.167A1.167 1.167 0 0 1 1 15.833V4.167A1.166 1.166 0 0 1 2.167 3h4.618m4.447-2H17v5.768M9.111 8.889l7.778-7.778"></path></svg>`;
});
const IconInfo = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  return `<div class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-blue-100 text-blue-500 dark:bg-blue-800 dark:text-blue-200">${validate_component(InfoCircleSolid, "InfoCircleSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})}</div>`;
});
const IconSuccess = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  return `<div class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-green-100 text-green-500 dark:bg-green-800 dark:text-green-200">${validate_component(CheckCircleSolid, "CheckCircleSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})}</div>`;
});
const RaindexClientProvider = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { raindexClient } = $$props;
  setContext(RAINDEX_CLIENT_CONTEXT_KEY, raindexClient);
  if ($$props.raindexClient === void 0 && $$bindings.raindexClient && raindexClient !== void 0) $$bindings.raindexClient(raindexClient);
  return `${slots.default ? slots.default({}) : ``}`;
});
const WalletProvider = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { account = readable(null) } = $$props;
  setAccountContext(account);
  if ($$props.account === void 0 && $$bindings.account && account !== void 0) $$bindings.account(account);
  return `${slots.default ? slots.default({}) : ``}`;
});
const ToastDetail = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { toast } = $$props;
  let { i } = $$props;
  useToasts();
  if ($$props.toast === void 0 && $$bindings.toast && toast !== void 0) $$bindings.toast(toast);
  if ($$props.i === void 0 && $$bindings.i && i !== void 0) $$bindings.i(i);
  return `${validate_component(Toast, "Toast").$$render(
    $$result,
    {
      dismissable: true,
      transition: slide,
      color: toast.color,
      class: "mb-2"
    },
    {},
    {
      icon: () => {
        return `${toast.type === "success" ? `${validate_component(CheckCircleSolid, "CheckCircleSolid").$$render(
          $$result,
          {
            class: "h-5 w-5",
            "data-testid": "success-icon"
          },
          {},
          {}
        )}` : `${toast.type === "error" ? `${validate_component(CloseCircleSolid, "CloseCircleSolid").$$render(
          $$result,
          {
            class: "h-5 w-5",
            "data-testid": "error-icon"
          },
          {},
          {}
        )}` : ``}`} `;
      },
      default: () => {
        return `<p class="font-semibold">${escape(toast.message)}</p> ${toast.detail ? `<p>${escape(toast.detail)}</p>` : ``} ${toast.links ? `<div class="flex flex-col">${each(toast.links, ({ link, label }) => {
          return `<a${add_attribute("href", link, 0)} target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">${escape(label)} </a>`;
        })}</div>` : ``}`;
      }
    }
  )}`;
});
const ToastProvider = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $toasts, $$unsubscribe_toasts;
  const toasts = writable([]);
  $$unsubscribe_toasts = subscribe(toasts, (value) => $toasts = value);
  setToastsContext(toasts);
  $$unsubscribe_toasts();
  return `<div class="fixed right-4 top-4 z-[100]">${each($toasts, (toast, i) => {
    return `<div data-testid="toast">${validate_component(ToastDetail, "ToastDetail").$$render($$result, { toast, i }, {}, {})} </div>`;
  })}</div> ${slots.default ? slots.default({}) : ``}`;
});
const ButtonDarkMode = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { colorTheme: colorTheme2 = writable() } = $$props;
  if ($$props.colorTheme === void 0 && $$bindings.colorTheme && colorTheme2 !== void 0) $$bindings.colorTheme(colorTheme2);
  return `<button type="button">${validate_component(DarkMode, "DarkMode").$$render($$result, {}, {}, {})}</button>`;
});
const ModalConnect = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let label;
  let $ledgerWalletAddress, $$unsubscribe_ledgerWalletAddress;
  let $walletconnectAccount, $$unsubscribe_walletconnectAccount;
  $$unsubscribe_ledgerWalletAddress = subscribe(ledgerWalletAddress, (value) => $ledgerWalletAddress = value);
  $$unsubscribe_walletconnectAccount = subscribe(walletconnectAccount, (value) => $walletconnectAccount = value);
  let open = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;
  function reset() {
    open = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    label = $walletconnectAccount ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}` : $ledgerWalletAddress ? `${$ledgerWalletAddress.slice(0, 5)}...${$ledgerWalletAddress.slice(-5)}` : "Connect to Wallet";
    $$rendered = `<div class="flex w-full flex-col py-4">${validate_component(Button, "Button").$$render($$result, { color: "primary", pill: true }, {}, {
      default: () => {
        return `${escape(label)}`;
      }
    })}</div> ${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: "Connect to Wallet",
        outsideclose: true,
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
          return `${!selectedLedger && !selectedWalletconnect && !$walletconnectAccount && !$ledgerWalletAddress ? `<div class="flex justify-center space-x-4">${validate_component(Button, "Button").$$render($$result, { class: "text-lg" }, {}, {
            default: () => {
              return `<div class="mr-4">${validate_component(IconLedger, "IconLedger").$$render($$result, {}, {}, {})}</div>
        Ledger Wallet`;
            }
          })} ${validate_component(Button, "Button").$$render($$result, { class: "text-lg" }, {}, {
            default: () => {
              return `<div class="mr-3">${validate_component(IconWalletConnect, "IconWalletConnect").$$render($$result, {}, {}, {})}</div>
        WalletConnect`;
            }
          })}</div>` : `${selectedLedger || $ledgerWalletAddress ? `${validate_component(InputLedgerWallet, "InputLedgerWallet").$$render($$result, { onConnect: reset }, {}, {})} ${!$ledgerWalletAddress ? `<div class="flex justify-between space-x-4">${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Back`;
            }
          })}</div>` : ``}` : `${selectedWalletconnect || $walletconnectAccount ? `${validate_component(InputWalletConnect, "InputWalletConnect").$$render($$result, { onConnect: reset }, {}, {})} ${!$walletconnectAccount ? `<div class="flex justify-between space-x-4">${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Back`;
            }
          })}</div>` : ``}` : ``}`}`}`;
        }
      }
    )}`;
  } while (!$$settled);
  $$unsubscribe_ledgerWalletAddress();
  $$unsubscribe_walletconnectAccount();
  return $$rendered;
});
const Sidebar_1 = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  let $colorTheme, $$unsubscribe_colorTheme;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  $$unsubscribe_colorTheme = subscribe(colorTheme, (value) => $colorTheme = value);
  let app_sha;
  $$unsubscribe_page();
  $$unsubscribe_colorTheme();
  return `${validate_component(Sidebar, "Sidebar").$$render(
    $$result,
    {
      activeUrl: $page.url.pathname,
      asideClass: "w-64 fixed z-10"
    },
    {},
    {
      default: () => {
        return `${validate_component(SidebarWrapper, "SidebarWrapper").$$render(
          $$result,
          {
            divClass: "overflow-y-auto py-11 px-3 bg-gray-100 dark:bg-gray-800 min-h-screen"
          },
          {},
          {
            default: () => {
              return `${validate_component(SidebarGroup, "SidebarGroup").$$render($$result, { ulClass: "" }, {}, {
                default: () => {
                  return `<div class="block dark:hidden">${validate_component(SidebarBrand, "SidebarBrand").$$render(
                    $$result,
                    {
                      site: {
                        name: "",
                        href: "/",
                        img: $colorTheme === "dark" ? logoDark : logoLight
                      },
                      imgClass: "w-2/3 m-auto",
                      aClass: "w-full flex items-center justify-start gap-x-3 mb-5",
                      spanClass: "hidden"
                    },
                    {},
                    {}
                  )}</div> <div class="hidden dark:block">${validate_component(SidebarBrand, "SidebarBrand").$$render(
                    $$result,
                    {
                      site: {
                        name: "",
                        href: "/",
                        img: $colorTheme === "dark" ? logoDark : logoLight
                      },
                      imgClass: "w-2/3 m-auto",
                      aClass: "w-full flex items-center justify-start gap-x-3 mb-5",
                      spanClass: "hidden"
                    },
                    {},
                    {}
                  )}</div>`;
                }
              })} ${validate_component(SidebarGroup, "SidebarGroup").$$render($$result, { border: true }, {}, {
                default: () => {
                  return `${validate_component(SidebarItem, "SidebarItem").$$render($$result, { label: "New Order", href: "/orders/add" }, {}, {
                    icon: () => {
                      return `${validate_component(PlusSolid, "PlusSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})} <span data-testid="sidebar-new-order"></span>`;
                    }
                  })} ${validate_component(SidebarItem, "SidebarItem").$$render($$result, { label: "Orders", href: "/orders" }, {}, {
                    icon: () => {
                      return `${validate_component(ReceiptSolid, "ReceiptSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})} <span data-testid="sidebar-orders"></span>`;
                    }
                  })} ${validate_component(SidebarItem, "SidebarItem").$$render($$result, { label: "Vaults", href: "/vaults" }, {}, {
                    icon: () => {
                      return `${validate_component(WalletSolid, "WalletSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})} <span data-testid="sidebar-vaults"></span>`;
                    }
                  })}`;
                }
              })} ${validate_component(SidebarGroup, "SidebarGroup").$$render($$result, { border: true }, {}, {
                default: () => {
                  return `${validate_component(ModalConnect, "ModalConnect").$$render($$result, {}, {}, {})}`;
                }
              })} ${validate_component(SidebarGroup, "SidebarGroup").$$render($$result, { border: true }, {}, {
                default: () => {
                  return `${validate_component(SidebarItem, "SidebarItem").$$render($$result, { label: "Settings", href: "/settings" }, {}, {
                    icon: () => {
                      return `${validate_component(GearSolid, "GearSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})} <span data-testid="sidebar-settings"></span>`;
                    }
                  })} ${validate_component(SidebarItem, "SidebarItem").$$render(
                    $$result,
                    {
                      label: "Documentation",
                      target: "_blank",
                      href: "https://docs.rainlang.xyz/raindex/overview"
                    },
                    {},
                    {
                      icon: () => {
                        return `${validate_component(IconExternalLink, "IconExternalLink").$$render($$result, {}, {}, {})} <span data-testid="sidebar-documentation"></span>`;
                      }
                    }
                  )} ${validate_component(SidebarItem, "SidebarItem").$$render(
                    $$result,
                    {
                      label: "Ask for help",
                      target: "_blank",
                      href: "https://t.me/+W0aQ36ptN_E2MjZk"
                    },
                    {},
                    {
                      icon: () => {
                        return `${validate_component(IconTelegram, "IconTelegram").$$render($$result, {}, {}, {})} <span data-testid="sidebar-telegram"></span>`;
                      }
                    }
                  )} ${validate_component(SidebarItem, "SidebarItem").$$render($$result, { label: "License", href: "/license" }, {}, {
                    icon: () => {
                      return `${validate_component(FileLinesSolid, "FileLinesSolid").$$render($$result, {}, {}, {})} <span data-testid="sidebar-license"></span>`;
                    }
                  })}`;
                }
              })} ${validate_component(SidebarGroup, "SidebarGroup").$$render(
                $$result,
                {
                  border: true,
                  class: "flex justify-start"
                },
                {},
                {
                  default: () => {
                    return `${validate_component(ButtonDarkMode, "ButtonDarkMode").$$render($$result, { colorTheme }, {}, {})}`;
                  }
                }
              )} ${validate_component(SidebarGroup, "SidebarGroup").$$render(
                $$result,
                {
                  border: true,
                  class: "flex justify-start self-end"
                },
                {},
                {
                  default: () => {
                    return `<div class="flex flex-col text-xs text-gray-500 dark:text-gray-400"><p data-svelte-h="svelte-radz2y">Raindex version commit:</p> <p class="break-all">${escape(app_sha)}</p></div>`;
                  }
                }
              )}`;
            }
          }
        )}`;
      }
    }
  )}`;
});
const AppToast = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { toast } = $$props;
  let toastColor;
  function getToastColor() {
    if (toast.message_type === ToastMessageType.Success) {
      return "green";
    } else if (toast.message_type === ToastMessageType.Error) {
      return "red";
    } else if (toast.message_type === ToastMessageType.Warning) {
      return "yellow";
    } else if (toast.message_type === ToastMessageType.Info) {
      return "info";
    }
  }
  if ($$props.toast === void 0 && $$bindings.toast && toast !== void 0) $$bindings.toast(toast);
  {
    if (toast) getToastColor();
  }
  return `<div class="mt-2">${validate_component(Toast, "Toast").$$render(
    $$result,
    {
      color: toastColor,
      contentClass: "w-full text-sm font-normal flex justify-start space-x-4 items-center pr-8",
      divClass: "w-full max-w-xs p-2 text-gray-500 bg-white shadow dark:text-gray-400 dark:bg-gray-800 gap-3 relative"
    },
    {},
    {
      "close-button": ({ close }) => {
        return `${validate_component(CloseSolid, "CloseSolid").$$render(
          $$result,
          {
            slot: "close-button",
            class: "absolute right-2 top-2 h-3 w-3 hover:opacity-50"
          },
          {},
          {}
        )} `;
      },
      default: () => {
        return `${toast.message_type === ToastMessageType.Success ? `${validate_component(IconSuccess, "IconSuccess").$$render($$result, {}, {}, {})}` : `${toast.message_type === ToastMessageType.Error ? `${validate_component(IconError, "IconError").$$render($$result, {}, {}, {})}` : `${toast.message_type === ToastMessageType.Warning ? `${validate_component(IconWarning, "IconWarning").$$render($$result, {}, {}, {})}` : `${toast.message_type === ToastMessageType.Info ? `${validate_component(IconInfo, "IconInfo").$$render($$result, {}, {}, {})}` : ``}`}`}`} <div class="max-h-48 overflow-scroll">${escape(toast.text)}</div>`;
      }
    }
  )}</div>`;
});
function useTransactionStatusNoticeStore(autoCloseMs = 5e3) {
  const { subscribe: subscribe2, update } = writable({});
  listen(
    "transaction_status_notice",
    (event) => handleNotice(event.payload)
  );
  function handleNotice(payload) {
    update((val) => {
      val[payload.id] = { ...payload };
      return val;
    });
    if (payload.status.type === "Failed" || payload.status.type === "Confirmed") {
      setTimeout(() => {
        update((val) => {
          const newVal = { ...val };
          delete newVal[payload.id];
          return newVal;
        });
      }, autoCloseMs);
    }
  }
  return {
    subscribe: subscribe2
  };
}
const transactionStatusNotices = useTransactionStatusNoticeStore();
const transactionStatusNoticesList = derived(
  transactionStatusNotices,
  (obj) => sortBy(Object.values(obj), [(val) => new Date(val.created_at), (val) => val.id])
);
const TransactionStatusNotice = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { transactionStatusNotice } = $$props;
  if ($$props.transactionStatusNotice === void 0 && $$bindings.transactionStatusNotice && transactionStatusNotice !== void 0) $$bindings.transactionStatusNotice(transactionStatusNotice);
  return `${validate_component(Toast, "Toast").$$render(
    $$result,
    {
      class: "mt-2 w-full !max-w-none",
      dismissable: false
    },
    {},
    {
      default: () => {
        return `<div data-testid="notice-label" class="mb-4 text-lg font-bold text-gray-900 dark:text-white">${escape(transactionStatusNotice.label)}</div> <div class="flex w-full items-center justify-start space-x-4 px-4">${transactionStatusNotice.status.type === "Initialized" || transactionStatusNotice.status.type === "PendingPrepare" ? `${validate_component(Spinner, "Spinner").$$render($$result, { "data-testid": "status-pending-prepare" }, {}, {})} <div class="mb-2 text-xl" data-svelte-h="svelte-10w5ro7">Preparing Transaction</div>` : `${transactionStatusNotice.status.type === "PendingSign" ? `${validate_component(ExclamationCircleSolid, "ExclamationCircleSolid").$$render($$result, { class: "h-10 w-10", color: "yellow" }, {}, {})} <div data-testid="status-pending-sign" data-svelte-h="svelte-1khvmkg"><div class="mb-2 text-xl">Awaiting Signature</div> <div>Please review and sign the transaction on your Ledger device</div></div>` : `${transactionStatusNotice.status.type === "Sending" ? `${validate_component(Spinner, "Spinner").$$render($$result, { "data-testid": "status-sending" }, {}, {})} <div data-svelte-h="svelte-rz5etq"><div class="mb-2 text-xl">Submitting Transaction</div> <div>Sending and awaiting confirmations...</div></div>` : `${transactionStatusNotice.status.type === "Confirmed" ? `${validate_component(CheckCircleSolid, "CheckCircleSolid").$$render(
          $$result,
          {
            "data-testid": "status-confirmed",
            class: "h-10 w-10",
            color: "green"
          },
          {},
          {}
        )} <div><div class="mb-2 text-xl" data-svelte-h="svelte-1bbaut8">Transaction Confirmed</div> <div data-testid="confirmed-payload" class="mb-4 break-all">Hash: ${escape(transactionStatusNotice.status.payload)}</div> ${validate_component(Button, "Button").$$render(
          $$result,
          {
            "data-testid": "block-explorer-link",
            size: "xs",
            color: "light",
            href: formatBlockExplorerTransactionUrl(transactionStatusNotice.chain_id, transactionStatusNotice.status.payload),
            target: "_blank"
          },
          {},
          {
            default: () => {
              return `View on Block Explorer`;
            }
          }
        )}</div>` : `${transactionStatusNotice.status.type === "Failed" ? `${validate_component(CloseCircleSolid, "CloseCircleSolid").$$render(
          $$result,
          {
            "data-testid": "status-failed",
            class: "h-10 w-10",
            color: "red"
          },
          {},
          {}
        )} <div><div class="mb-2 text-xl" data-svelte-h="svelte-obyp6c">Transaction Failed</div> <div data-testid="failed-payload">${escape(transactionStatusNotice.status.payload)}</div></div>` : ``}`}`}`}`}</div>`;
      }
    }
  )}`;
});
const WindowDraggableArea = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let currentPlatform = void 0;
  async function setup() {
    currentPlatform = await platform();
  }
  setup();
  return ` ${currentPlatform === "darwin" ? ` <div class="fixed top-0 z-100 h-[28px] w-full" data-tauri-drag-region></div>` : ``}`;
});
const Layout = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $settingsText, $$unsubscribe_settingsText;
  let $transactionStatusNoticesList, $$unsubscribe_transactionStatusNoticesList;
  let $toastsList, $$unsubscribe_toastsList;
  $$unsubscribe_settingsText = subscribe(settingsText, (value) => $settingsText = value);
  $$unsubscribe_transactionStatusNoticesList = subscribe(transactionStatusNoticesList, (value) => $transactionStatusNoticesList = value);
  $$unsubscribe_toastsList = subscribe(toastsList, (value) => $toastsList = value);
  const account = derived([ledgerWalletAddress, walletconnectAccount], ([$ledgerWalletAddress, $walletconnectAccount]) => {
    return $ledgerWalletAddress || $walletconnectAccount || null;
  });
  let raindexClient = void 0;
  {
    if ($settingsText) {
      const result = RaindexClient.new([$settingsText]);
      if (result.error) {
        throw new Error(result.error.readableMsg);
      }
      raindexClient = result.value;
      const uniqueChainIds = raindexClient.getUniqueChainIds();
      if (!uniqueChainIds.error) {
        validChainIds.set(uniqueChainIds.value);
      }
    }
  }
  $$unsubscribe_settingsText();
  $$unsubscribe_transactionStatusNoticesList();
  $$unsubscribe_toastsList();
  return `${validate_component(WindowDraggableArea, "WindowDraggableArea").$$render($$result, {}, {}, {})} ${validate_component(ToastProvider, "ToastProvider").$$render($$result, {}, {}, {
    default: () => {
      return `${validate_component(WalletProvider, "WalletProvider").$$render($$result, { account }, {}, {
        default: () => {
          return `${validate_component(QueryClientProvider, "QueryClientProvider").$$render($$result, { client: queryClient }, {}, {
            default: () => {
              return `${raindexClient ? `${validate_component(RaindexClientProvider, "RaindexClientProvider").$$render($$result, { raindexClient }, {}, {
                default: () => {
                  return `<div class="mb-10 flex h-[calc(100vh-2.5rem)] w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">${validate_component(Sidebar_1, "Sidebar").$$render($$result, {}, {}, {})} <main class="ml-64 h-full w-full grow overflow-x-auto p-8">${slots.default ? slots.default({}) : ``}</main> <div class="fixed right-5 top-5 z-50 w-full max-w-md">${each($transactionStatusNoticesList, (transactionStatusNotice) => {
                    return `${validate_component(TransactionStatusNotice, "TransactionStatusNotice").$$render($$result, { transactionStatusNotice }, {}, {})}`;
                  })} ${each($toastsList, (toast) => {
                    return `<div class="flex justify-end">${validate_component(AppToast, "AppToast").$$render($$result, { toast }, {}, {})} </div>`;
                  })}</div> <div class="fixed bottom-0 left-64 right-0 h-10 bg-primary-400 p-2 text-center text-white" data-svelte-h="svelte-1davhii">The Raindex app is still early alpha - have fun but use at your own risk!</div></div>`;
                }
              })}` : ``}`;
            }
          })}`;
        }
      })}`;
    }
  })}`;
});
export {
  Layout as default
};
//# sourceMappingURL=_layout.svelte.js.map
