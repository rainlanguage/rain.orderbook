import { c as create_ssr_component, a as compute_rest_props, l as createEventDispatcher, f as add_attribute, v as validate_component, b as spread, d as escape_object, e as escape_attribute_value, h as escape, i as compute_slots, g as getContext, k as subscribe } from "./ssr.js";
import { twJoin, twMerge } from "tailwind-merge";
import { F as Frame, C as CloseButton, k as toasts, r as reportErrorToSentry, A as Alert, B as ButtonLoading } from "./sentry.js";
import "imask/esm";
import "imask";
import "imask/esm/imask";
import "@tauri-apps/api";
import "@sentry/sveltekit";
import "@tauri-apps/api/os";
import "@tauri-apps/api/app";
import { writable as writable$1, get } from "@square/svelte-store";
import { c as colorTheme } from "./darkMode.js";
import Provider from "@walletconnect/ethereum-provider";
import { isHex, hexToNumber } from "viem";
import { w as writable } from "./index.js";
import * as dom from "@floating-ui/dom";
import * as chains from "viem/chains";
import { B as BROWSER } from "./node.js";
import { QueryClient } from "@tanstack/svelte-query";
const browser = BROWSER;
const Popper = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let middleware;
  let $$restProps = compute_rest_props($$props, [
    "activeContent",
    "arrow",
    "offset",
    "placement",
    "trigger",
    "triggeredBy",
    "reference",
    "strategy",
    "open",
    "yOnly",
    "middlewares"
  ]);
  let { activeContent = false } = $$props;
  let { arrow = true } = $$props;
  let { offset = 8 } = $$props;
  let { placement = "top" } = $$props;
  let { trigger = "hover" } = $$props;
  let { triggeredBy = void 0 } = $$props;
  let { reference = void 0 } = $$props;
  let { strategy = "absolute" } = $$props;
  let { open = false } = $$props;
  let { yOnly = false } = $$props;
  let { middlewares = [dom.flip(), dom.shift()] } = $$props;
  const dispatch = createEventDispatcher();
  let referenceEl;
  let floatingEl;
  let arrowEl;
  let contentEl;
  const px = (n) => n != null ? `${n}px` : "";
  let arrowSide;
  const oppositeSideMap = {
    left: "right",
    right: "left",
    bottom: "top",
    top: "bottom"
  };
  function updatePosition() {
    dom.computePosition(referenceEl, floatingEl, { placement, strategy, middleware }).then(({ x, y, middlewareData, placement: placement2, strategy: strategy2 }) => {
      floatingEl.style.position = strategy2;
      floatingEl.style.left = yOnly ? "0" : px(x);
      floatingEl.style.top = px(y);
      if (middlewareData.arrow && arrowEl instanceof HTMLDivElement) {
        arrowEl.style.left = px(middlewareData.arrow.x);
        arrowEl.style.top = px(middlewareData.arrow.y);
        arrowSide = oppositeSideMap[placement2.split("-")[0]];
        arrowEl.style[arrowSide] = px(-arrowEl.offsetWidth / 2 - ($$props.border ? 1 : 0));
      }
    });
  }
  function init(node, _referenceEl) {
    floatingEl = node;
    let cleanup = dom.autoUpdate(_referenceEl, floatingEl, updatePosition);
    return {
      update(_referenceEl2) {
        cleanup();
        cleanup = dom.autoUpdate(_referenceEl2, floatingEl, updatePosition);
      },
      destroy() {
        cleanup();
      }
    };
  }
  let arrowClass;
  if ($$props.activeContent === void 0 && $$bindings.activeContent && activeContent !== void 0) $$bindings.activeContent(activeContent);
  if ($$props.arrow === void 0 && $$bindings.arrow && arrow !== void 0) $$bindings.arrow(arrow);
  if ($$props.offset === void 0 && $$bindings.offset && offset !== void 0) $$bindings.offset(offset);
  if ($$props.placement === void 0 && $$bindings.placement && placement !== void 0) $$bindings.placement(placement);
  if ($$props.trigger === void 0 && $$bindings.trigger && trigger !== void 0) $$bindings.trigger(trigger);
  if ($$props.triggeredBy === void 0 && $$bindings.triggeredBy && triggeredBy !== void 0) $$bindings.triggeredBy(triggeredBy);
  if ($$props.reference === void 0 && $$bindings.reference && reference !== void 0) $$bindings.reference(reference);
  if ($$props.strategy === void 0 && $$bindings.strategy && strategy !== void 0) $$bindings.strategy(strategy);
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.yOnly === void 0 && $$bindings.yOnly && yOnly !== void 0) $$bindings.yOnly(yOnly);
  if ($$props.middlewares === void 0 && $$bindings.middlewares && middlewares !== void 0) $$bindings.middlewares(middlewares);
  {
    dispatch("show", open);
  }
  placement && (referenceEl = referenceEl);
  middleware = [
    ...middlewares,
    dom.offset(+offset),
    arrowEl
  ];
  arrowClass = twJoin("absolute pointer-events-none block w-[10px] h-[10px] rotate-45 bg-inherit border-inherit", $$props.border && arrowSide === "bottom" && "border-b border-e", $$props.border && arrowSide === "top" && "border-t border-s ", $$props.border && arrowSide === "right" && "border-t border-e ", $$props.border && arrowSide === "left" && "border-b border-s ");
  return `${!referenceEl ? `<div${add_attribute("this", contentEl, 0)}></div>` : ``} ${open && referenceEl ? `${validate_component(Frame, "Frame").$$render($$result, Object.assign({}, { use: init }, { options: referenceEl }, { role: "tooltip" }, { tabindex: activeContent ? -1 : void 0 }, $$restProps), {}, {
    default: () => {
      return `${slots.default ? slots.default({}) : ``} ${arrow ? `<div${add_attribute("class", arrowClass, 0)}></div>` : ``}`;
    }
  })}` : ``} `;
});
const Helper = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["helperClass", "color"]);
  let { helperClass = "text-xs font-normal text-gray-500 dark:text-gray-300" } = $$props;
  let { color = "gray" } = $$props;
  const colorClasses = {
    gray: "text-gray-900 dark:text-gray-300",
    green: "text-green-700 dark:text-green-500",
    red: "text-red-700 dark:text-red-500",
    disabled: "text-gray-400 dark:text-gray-500"
  };
  if ($$props.helperClass === void 0 && $$bindings.helperClass && helperClass !== void 0) $$bindings.helperClass(helperClass);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  return `<p${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(helperClass, colorClasses[color], $$props.class))
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</p> `;
});
const Modal = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, [
    "open",
    "title",
    "size",
    "placement",
    "autoclose",
    "dismissable",
    "backdropClass",
    "defaultClass",
    "outsideclose",
    "dialogClass"
  ]);
  let $$slots = compute_slots(slots);
  let { open = false } = $$props;
  let { title = "" } = $$props;
  let { size = "md" } = $$props;
  let { placement = "center" } = $$props;
  let { autoclose = false } = $$props;
  let { dismissable = true } = $$props;
  let { backdropClass = "fixed inset-0 z-40 bg-gray-900 bg-opacity-50 dark:bg-opacity-80" } = $$props;
  let { defaultClass = "relative flex flex-col mx-auto" } = $$props;
  let { outsideclose = false } = $$props;
  let { dialogClass = "fixed top-0 start-0 end-0 h-modal md:inset-0 md:h-full z-50 w-full p-4 flex" } = $$props;
  const dispatch = createEventDispatcher();
  const getPlacementClasses = () => {
    switch (placement) {
      case "top-left":
        return ["justify-start", "items-start"];
      case "top-center":
        return ["justify-center", "items-start"];
      case "top-right":
        return ["justify-end", "items-start"];
      case "center-left":
        return ["justify-start", "items-center"];
      case "center":
        return ["justify-center", "items-center"];
      case "center-right":
        return ["justify-end", "items-center"];
      case "bottom-left":
        return ["justify-start", "items-end"];
      case "bottom-center":
        return ["justify-center", "items-end"];
      case "bottom-right":
        return ["justify-end", "items-end"];
      default:
        return ["justify-center", "items-center"];
    }
  };
  const sizes = {
    xs: "max-w-md",
    sm: "max-w-lg",
    md: "max-w-2xl",
    lg: "max-w-4xl",
    xl: "max-w-7xl"
  };
  let frameClass;
  let backdropCls = twMerge(backdropClass, $$props.classBackdrop);
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.title === void 0 && $$bindings.title && title !== void 0) $$bindings.title(title);
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.placement === void 0 && $$bindings.placement && placement !== void 0) $$bindings.placement(placement);
  if ($$props.autoclose === void 0 && $$bindings.autoclose && autoclose !== void 0) $$bindings.autoclose(autoclose);
  if ($$props.dismissable === void 0 && $$bindings.dismissable && dismissable !== void 0) $$bindings.dismissable(dismissable);
  if ($$props.backdropClass === void 0 && $$bindings.backdropClass && backdropClass !== void 0) $$bindings.backdropClass(backdropClass);
  if ($$props.defaultClass === void 0 && $$bindings.defaultClass && defaultClass !== void 0) $$bindings.defaultClass(defaultClass);
  if ($$props.outsideclose === void 0 && $$bindings.outsideclose && outsideclose !== void 0) $$bindings.outsideclose(outsideclose);
  if ($$props.dialogClass === void 0 && $$bindings.dialogClass && dialogClass !== void 0) $$bindings.dialogClass(dialogClass);
  {
    dispatch(open ? "open" : "close");
  }
  frameClass = twMerge(defaultClass, "w-full divide-y", $$props.class);
  return `${open ? ` <div${add_attribute("class", backdropCls, 0)}></div>   <div${add_attribute("class", twMerge(dialogClass, $$props.classDialog, ...getPlacementClasses()), 0)} tabindex="-1" aria-modal="true" role="dialog"><div class="${"flex relative " + escape(sizes[size], true) + " w-full max-h-full"}"> ${validate_component(Frame, "Frame").$$render($$result, Object.assign({}, { rounded: true }, { shadow: true }, $$restProps, { class: frameClass }), {}, {
    default: () => {
      return ` ${$$slots.header || title ? `${validate_component(Frame, "Frame").$$render(
        $$result,
        {
          color: $$restProps.color,
          class: "flex justify-between items-center p-4 md:p-5 rounded-t-lg"
        },
        {},
        {
          default: () => {
            return `${slots.header ? slots.header({}) : ` <h3 class="${"text-xl font-semibold " + escape($$restProps.color ? "" : "text-gray-900 dark:text-white", true) + " p-0"}">${escape(title)}</h3> `} ${dismissable ? `${validate_component(CloseButton, "CloseButton").$$render(
              $$result,
              {
                name: "Close modal",
                color: $$restProps.color
              },
              {},
              {}
            )}` : ``}`;
          }
        }
      )}` : ``}  <div${add_attribute("class", twMerge("p-4 md:p-5 space-y-4 flex-1 overflow-y-auto overscroll-contain", $$props.bodyClass), 0)} role="document">${dismissable && !$$slots.header && !title ? `${validate_component(CloseButton, "CloseButton").$$render(
        $$result,
        {
          name: "Close modal",
          class: "absolute top-3 end-2.5",
          color: $$restProps.color
        },
        {},
        {}
      )}` : ``} ${slots.default ? slots.default({}) : ``}</div>  ${$$slots.footer ? `${validate_component(Frame, "Frame").$$render(
        $$result,
        {
          color: $$restProps.color,
          class: "flex items-center p-4 md:p-5 space-x-3 rtl:space-x-reverse rounded-b-lg"
        },
        {},
        {
          default: () => {
            return `${slots.footer ? slots.footer({}) : ``}`;
          }
        }
      )}` : ``}`;
    }
  })}</div></div>` : ``} `;
});
const Tooltip = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["type", "defaultClass"]);
  let { type = "dark" } = $$props;
  let { defaultClass = "py-2 px-3 text-sm font-medium" } = $$props;
  const types = {
    dark: "bg-gray-900 text-white dark:bg-gray-700",
    light: "border-gray-200 bg-white text-gray-900",
    auto: " bg-white text-gray-900 dark:bg-gray-700 dark:text-white border-gray-200 dark:border-gray-700",
    custom: ""
  };
  let toolTipClass;
  if ($$props.type === void 0 && $$bindings.type && type !== void 0) $$bindings.type(type);
  if ($$props.defaultClass === void 0 && $$bindings.defaultClass && defaultClass !== void 0) $$bindings.defaultClass(defaultClass);
  {
    {
      if ($$restProps.color) type = "custom";
      else $$restProps.color = "none";
      if (["light", "auto"].includes(type)) $$restProps.border = true;
      toolTipClass = twMerge("tooltip", defaultClass, types[type], $$props.class);
    }
  }
  return `${validate_component(Popper, "Popper").$$render($$result, Object.assign({}, { rounded: true }, { shadow: true }, $$restProps, { class: toolTipClass }), {}, {
    default: () => {
      return `${slots.default ? slots.default({}) : ``}`;
    }
  })} `;
});
const Tooltip_1 = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { triggeredBy = "" } = $$props;
  let { customClass = "" } = $$props;
  if ($$props.triggeredBy === void 0 && $$bindings.triggeredBy && triggeredBy !== void 0) $$bindings.triggeredBy(triggeredBy);
  if ($$props.customClass === void 0 && $$bindings.customClass && customClass !== void 0) $$bindings.customClass(customClass);
  return `${validate_component(Tooltip, "Tooltip").$$render(
    $$result,
    {
      triggeredBy,
      class: `z-[99] shadow-none ${customClass}`
    },
    {},
    {
      default: () => {
        return `${slots.default ? slots.default({}) : ``}`;
      }
    }
  )}`;
});
const ClipboardListOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "clipboard list outline" } = $$props;
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.role === void 0 && $$bindings.role && role !== void 0) $$bindings.role(role);
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
      { viewBox: "0 0 18 20" }
    ],
    {}
  )}><g fill="currentColor"><path d="M16 1h-3.278A1.992 1.992 0 0 0 11 0H7a1.993 1.993 0 0 0-1.722 1H2a2 2 0 0 0-2 2v15a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V3a2 2 0 0 0-2-2Zm-5 1v2H7V2h4Zm5 16H2V3h3v1a1 1 0 0 0 0 2h8a1 1 0 1 0 0-2V3h3v15Z"></path><path d="M13 9H8a1 1 0 0 0 0 2h5a1 1 0 0 0 0-2Zm0 4H8a1 1 0 0 0 0 2h5a1 1 0 0 0 0-2Zm-8-2a1 1 0 1 0 0-2 1 1 0 0 0 0 2Zm0 4a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"></path></g></svg> `;
});
const ClipboardOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "clipboard outline" } = $$props;
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
      { viewBox: "0 0 18 20" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M12 2h4a1 1 0 0 1 1 1v15a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1h4m6 0a1 1 0 0 0-1-1H7a1 1 0 0 0-1 1m6 0v3H6V2M5 5h8m-8 5h8m-8 4h8"></path></svg> `;
});
const ExclamationCircleSolid = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "exclamation circle solid" } = $$props;
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
  )}><path fill="currentColor" d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5ZM10 15a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm1-4a1 1 0 0 1-2 0V6a1 1 0 0 1 2 0v5Z"></path></svg> `;
});
const FingerprintOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "fingerprint outline" } = $$props;
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
      { viewBox: "0 0 22 20" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M20 10a28.076 28.076 0 0 1-1.091 9M6.231 2.37a8.994 8.994 0 0 1 12.88 3.73M1.958 13S2 12.577 2 10a8.949 8.949 0 0 1 1.735-5.307m12.84 3.088A5.98 5.98 0 0 1 17 10a30 30 0 0 1-.464 6.232M5 10a6 6 0 0 1 9.352-4.974M3 19a5.964 5.964 0 0 1 1.01-3.328 5.15 5.15 0 0 0 .786-1.926m8.66 2.486a13.96 13.96 0 0 1-.962 2.683M6.5 17.336C8 15.092 8 12.845 8 10a3 3 0 1 1 6 0c0 .749 0 1.521-.031 2.311M11 10c0 3 0 6-2 9"></path></svg> `;
});
const WalletOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "wallet outline" } = $$props;
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
      { viewBox: "0 0 20 20" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M11.905 1.316 15.633 6M18 10h-5a2 2 0 0 0-2 2v1a2 2 0 0 0 2 2h5m0-5a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1m0-5V7a1 1 0 0 0-1-1H2a1 1 0 0 0-1 1v11a1 1 0 0 0 1 1h15a1 1 0 0 0 1-1v-3m-6.367-9L7.905 1.316 2.352 6h9.281Z"></path></svg> `;
});
var HashType = /* @__PURE__ */ ((HashType2) => {
  HashType2[HashType2["Identifier"] = 0] = "Identifier";
  HashType2[HashType2["Wallet"] = 1] = "Wallet";
  HashType2[HashType2["Transaction"] = 2] = "Transaction";
  HashType2[HashType2["Address"] = 3] = "Address";
  return HashType2;
})(HashType || {});
const Hash = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let id;
  let displayValue;
  let { value } = $$props;
  let { type = void 0 } = $$props;
  let { shorten = true } = $$props;
  let { sliceLen = 5 } = $$props;
  let { copyOnClick = true } = $$props;
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.type === void 0 && $$bindings.type && type !== void 0) $$bindings.type(type);
  if ($$props.shorten === void 0 && $$bindings.shorten && shorten !== void 0) $$bindings.shorten(shorten);
  if ($$props.sliceLen === void 0 && $$bindings.sliceLen && sliceLen !== void 0) $$bindings.sliceLen(sliceLen);
  if ($$props.copyOnClick === void 0 && $$bindings.copyOnClick && copyOnClick !== void 0) $$bindings.copyOnClick(copyOnClick);
  id = shorten ? `hash-${value}` : void 0;
  displayValue = value && shorten ? `${value.slice(0, sliceLen)}...${value.slice(-1 * sliceLen)}` : value;
  return `<button type="button"${add_attribute("id", id, 0)} class="flex items-center justify-start space-x-2 text-left">${type === HashType.Wallet ? `${validate_component(WalletOutline, "WalletOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Identifier ? `${validate_component(FingerprintOutline, "FingerprintOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Transaction ? `${validate_component(ClipboardListOutline, "ClipboardListOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Address ? `${validate_component(ClipboardOutline, "ClipboardOutline").$$render($$result, { size: "sm" }, {}, {})}` : ``}`}`}`} <div>${escape(displayValue)}</div></button> ${``} ${shorten ? `${validate_component(Tooltip_1, "Tooltip").$$render($$result, { triggeredBy: `#${id}` }, {}, {
    default: () => {
      return `<div class="flex items-center justify-start space-x-2">${type === HashType.Wallet ? `${validate_component(WalletOutline, "WalletOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Identifier ? `${validate_component(FingerprintOutline, "FingerprintOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Transaction ? `${validate_component(ClipboardListOutline, "ClipboardListOutline").$$render($$result, { size: "sm" }, {}, {})}` : `${type === HashType.Address ? `${validate_component(ClipboardOutline, "ClipboardOutline").$$render($$result, { size: "sm" }, {}, {})}` : ``}`}`}`} <div>${escape(value)}</div></div>`;
    }
  })}` : ``}`;
});
var DeploymentStepsErrorCode;
(function(DeploymentStepsErrorCode2) {
  DeploymentStepsErrorCode2["NO_GUI_PROVIDER"] = "No GUI provider found.";
  DeploymentStepsErrorCode2["NO_GUI"] = "Error loading GUI.";
  DeploymentStepsErrorCode2["NO_LOCAL_DB_PROVIDER"] = "No Local DB provider found.";
  DeploymentStepsErrorCode2["NO_STRATEGY"] = "No valid order exists at this URL";
  DeploymentStepsErrorCode2["NO_SELECT_TOKENS"] = "Error loading tokens";
  DeploymentStepsErrorCode2["NO_TOKEN_INFO"] = "Error loading token information";
  DeploymentStepsErrorCode2["NO_FIELD_DEFINITIONS"] = "Error loading field definitions";
  DeploymentStepsErrorCode2["NO_DEPOSITS"] = "Error loading deposits";
  DeploymentStepsErrorCode2["NO_TOKEN_INPUTS"] = "Error loading token inputs";
  DeploymentStepsErrorCode2["NO_TOKEN_OUTPUTS"] = "Error loading token outputs";
  DeploymentStepsErrorCode2["NO_GUI_DETAILS"] = "Error getting GUI details";
  DeploymentStepsErrorCode2["NO_CHAIN"] = "Unsupported chain ID";
  DeploymentStepsErrorCode2["NO_NETWORK_KEY"] = "No network key found";
  DeploymentStepsErrorCode2["NO_AVAILABLE_TOKENS"] = "Error loading available tokens";
  DeploymentStepsErrorCode2["SERIALIZE_ERROR"] = "Error serializing state";
  DeploymentStepsErrorCode2["ADD_ORDER_FAILED"] = "Failed to add order";
  DeploymentStepsErrorCode2["NO_WALLET"] = "No account address found";
  DeploymentStepsErrorCode2["NO_GUI_CONFIG"] = "Error getting GUI configuration";
  DeploymentStepsErrorCode2["NO_RAINDEX_CLIENT_PROVIDER"] = "No Raindex client provider found";
})(DeploymentStepsErrorCode || (DeploymentStepsErrorCode = {}));
class DeploymentStepsError extends Error {
  code;
  details;
  static errorStore = writable(null);
  constructor(code, details) {
    super(code);
    this.code = code;
    this.details = details;
    this.name = "DeploymentStepsError";
  }
  static get error() {
    return this.errorStore;
  }
  static throwIfNull(value, code) {
    if (value === null || value === void 0) {
      throw new DeploymentStepsError(code);
    }
    return value;
  }
  static catch(e, code) {
    const error = e instanceof DeploymentStepsError ? e : new DeploymentStepsError(code, e instanceof Error ? e.message : "Unknown error");
    this.errorStore.set(error);
  }
  static clear() {
    this.errorStore.set(null);
  }
}
const RAINDEX_CLIENT_CONTEXT_KEY = "raindex-client-context";
function useRaindexClient() {
  const raindexClient = getContext(RAINDEX_CLIENT_CONTEXT_KEY);
  if (!raindexClient) {
    DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_RAINDEX_CLIENT_PROVIDER);
  }
  return raindexClient;
}
const icon$1 = '<svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" x="0px" y="0px"\n	 viewBox="0 0 768.91 669.35" style="enable-background:new 0 0 768.91 669.35;" xml:space="preserve">\n<path d="M0,479.29v190.06h289.22V627.2H42.14V479.29H0z M726.77,479.29V627.2H479.69v42.14h289.22V479.29H726.77z M289.64,190.06\n	v289.22h190.05v-38.01H331.78V190.06H289.64z M0,0v190.06h42.14V42.14h247.08V0H0z M479.69,0v42.14h247.08v147.92h42.14V0H479.69z"\n	/>\n</svg>\n';
const css$1 = {
  code: ".icon-ledger svg{height:1.5rem;width:auto;fill:white}",
  map: '{"version":3,"file":"IconLedger.svelte","sources":["IconLedger.svelte"],"sourcesContent":["<script>import icon from \\"../assets/ledger.svg?raw\\";\\n<\/script>\\n\\n<!-- eslint-disable-next-line svelte/no-at-html-tags -->\\n<div class=\\"icon-ledger\\">{@html icon}</div>\\n\\n<style>\\n\\t:global(.icon-ledger svg) {\\n\\t\\theight: 1.5rem;\\n\\t\\twidth: auto;\\n\\t\\tfill: white;\\n\\t}\\n</style>\\n"],"names":[],"mappings":"AAOS,gBAAkB,CACzB,MAAM,CAAE,MAAM,CACd,KAAK,CAAE,IAAI,CACX,IAAI,CAAE,KACP"}'
};
const IconLedger = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  $$result.css.add(css$1);
  return ` <div class="icon-ledger"><!-- HTML_TAG_START -->${icon$1}<!-- HTML_TAG_END --></div>`;
});
const icon = '<svg fill="none" height="332" viewBox="0 0 480 332" width="480" xmlns="http://www.w3.org/2000/svg"><path d="m126.613 93.9842c62.622-61.3123 164.152-61.3123 226.775 0l7.536 7.3788c3.131 3.066 3.131 8.036 0 11.102l-25.781 25.242c-1.566 1.533-4.104 1.533-5.67 0l-10.371-10.154c-43.687-42.7734-114.517-42.7734-158.204 0l-11.107 10.874c-1.565 1.533-4.103 1.533-5.669 0l-25.781-25.242c-3.132-3.066-3.132-8.036 0-11.102zm280.093 52.2038 22.946 22.465c3.131 3.066 3.131 8.036 0 11.102l-103.463 101.301c-3.131 3.065-8.208 3.065-11.339 0l-73.432-71.896c-.783-.767-2.052-.767-2.835 0l-73.43 71.896c-3.131 3.065-8.208 3.065-11.339 0l-103.4657-101.302c-3.1311-3.066-3.1311-8.036 0-11.102l22.9456-22.466c3.1311-3.065 8.2077-3.065 11.3388 0l73.4333 71.897c.782.767 2.051.767 2.834 0l73.429-71.897c3.131-3.065 8.208-3.065 11.339 0l73.433 71.897c.783.767 2.052.767 2.835 0l73.431-71.895c3.132-3.066 8.208-3.066 11.339 0z" fill="#fff"/></svg>';
const css = {
  code: ".icon-walletconnect svg{height:1.5rem;width:auto;fill:white}",
  map: '{"version":3,"file":"IconWalletConnect.svelte","sources":["IconWalletConnect.svelte"],"sourcesContent":["<script>import icon from \\"../assets/walletconnect.svg?raw\\";\\n<\/script>\\n\\n<!-- eslint-disable-next-line svelte/no-at-html-tags -->\\n<div class=\\"icon-walletconnect\\">{@html icon}</div>\\n\\n<style>\\n\\t:global(.icon-walletconnect svg) {\\n\\t\\theight: 1.5rem;\\n\\t\\twidth: auto;\\n\\t\\tfill: white;\\n\\t}\\n</style>\\n"],"names":[],"mappings":"AAOS,uBAAyB,CAChC,MAAM,CAAE,MAAM,CACd,KAAK,CAAE,IAAI,CACX,IAAI,CAAE,KACP"}'
};
const IconWalletConnect = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  $$result.css.add(css);
  return ` <div class="icon-walletconnect"><!-- HTML_TAG_START -->${icon}<!-- HTML_TAG_END --></div>`;
});
const IconWarning = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  return `<div class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-yellow-100 text-yellow-500 dark:bg-yellow-800 dark:text-yellow-200">${validate_component(ExclamationCircleSolid, "ExclamationCircleSolid").$$render($$result, { class: "h-5 w-5" }, {}, {})}</div>`;
});
const ledgerWalletAddress = writable$1(null);
const ledgerWalletDerivationIndex = writable$1(0);
const WALLETCONNECT_PROJECT_ID = void 0;
const metadata = {
  name: "Raindex",
  description: "Raindex allows anyone to write, test, deploy and manage token trading orders written in rainlang, on any EVM network.",
  url: "https://rainlang.xyz",
  icons: [
    "https://raw.githubusercontent.com/rainlanguage/rain.brand/main/Raindex%20logo.svg",
    "https://raw.githubusercontent.com/WalletConnect/walletconnect-assets/master/Logo/Blue%20(Default)/Logo.svg"
  ]
};
const walletconnectAccount = writable$1(null);
const walletconnectIsDisconnecting = writable$1(false);
const walletconnectIsConnecting = writable$1(false);
let walletconnectProvider;
const walletConnectNetwork = writable$1(0);
Provider.init({
  metadata,
  projectId: WALLETCONNECT_PROJECT_ID,
  optionalChains: [1],
  optionalEvents: ["chainChanged", "accountsChanged", "connect", "disconnect"],
  showQrModal: true,
  qrModalOptions: {
    themeMode: get(colorTheme),
    enableExplorer: false
  }
}).then(async (provider) => {
  provider.on("connect", () => {
    walletconnectAccount.set(provider?.accounts?.[0] ?? null);
  });
  provider.on("disconnect", () => {
    walletconnectAccount.set(null);
  });
  provider.on("accountsChanged", (accounts) => {
    walletconnectAccount.set(accounts?.[0] ?? null);
  });
  provider.on("chainChanged", (chain) => {
    if (isHex(chain)) walletConnectNetwork.set(hexToNumber(chain));
    else walletConnectNetwork.set(parseInt(chain));
  });
  walletconnectProvider = provider;
  if (provider.accounts.length) {
    await walletconnectDisconnect();
  }
}).catch((e) => {
  toasts.error("Could not instantiate Walletconnect modal");
  reportErrorToSentry(e);
});
async function walletconnectDisconnect() {
  walletconnectIsDisconnecting.set(true);
  try {
    await walletconnectProvider?.disconnect();
  } catch (e) {
    reportErrorToSentry(e);
  }
  walletconnectIsDisconnecting.set(false);
  walletconnectAccount.set(null);
}
colorTheme.subscribe(
  (v) => walletconnectProvider?.modal?.setTheme({ themeMode: v })
);
const InputLedgerWallet = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$unsubscribe_walletConnectNetwork;
  let $ledgerWalletAddress, $$unsubscribe_ledgerWalletAddress;
  $$unsubscribe_walletConnectNetwork = subscribe(walletConnectNetwork, (value) => value);
  $$unsubscribe_ledgerWalletAddress = subscribe(ledgerWalletAddress, (value) => $ledgerWalletAddress = value);
  useRaindexClient();
  let { onConnect = () => {
  } } = $$props;
  let derivationIndex = 0;
  let isConnecting;
  let isDisconnecting = false;
  if ($$props.onConnect === void 0 && $$bindings.onConnect && onConnect !== void 0) $$bindings.onConnect(onConnect);
  $$unsubscribe_walletConnectNetwork();
  $$unsubscribe_ledgerWalletAddress();
  return `<div>${validate_component(Alert, "Alert").$$render(
    $$result,
    {
      color: "yellow",
      border: true,
      class: "mb-8"
    },
    {},
    {
      icon: () => {
        return `${validate_component(IconWarning, "IconWarning").$$render($$result, { slot: "icon" }, {}, {})}`;
      },
      default: () => {
        return `<div class="pl-2" data-svelte-h="svelte-31rzcn"><div class="mb-2 text-lg">Before you continue:</div> <ul role="list" class="list-disc space-y-2 pl-5"><li>All desktop applications linked to your Ledger wallet must be closed, including any
          desktop wallets and Ledger Live.</li> <li>Your Ledger wallet must be authenticated with the Ethereum app open.</li></ul></div>`;
      }
    }
  )} <div class="flex w-full items-start justify-end space-x-2">${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      color: "primary",
      class: "w-full px-2 py-1",
      size: "lg",
      pill: true,
      loading: isConnecting
    },
    {},
    {
      default: () => {
        return `${$ledgerWalletAddress ? `${validate_component(Hash, "Hash").$$render(
          $$result,
          {
            type: HashType.Wallet,
            value: $ledgerWalletAddress
          },
          {},
          {}
        )}` : `Connect`}`;
      }
    }
  )} ${$ledgerWalletAddress ? `${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      color: "red",
      class: "min-w-fit px-2 py-1",
      size: "lg",
      pill: true,
      loading: isDisconnecting
    },
    {},
    {
      default: () => {
        return `Disconnect`;
      }
    }
  )}` : `<div class="w-32 grow-0 break-all"><input type="text" class="block w-32 rounded-xl border-gray-300 bg-gray-50 p-1.5 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-primary-500 dark:focus:ring-primary-500 rtl:text-right"${add_attribute("value", derivationIndex, 0)}> ${validate_component(Helper, "Helper").$$render($$result, { class: "break-word mt-2 text-sm" }, {}, {
    default: () => {
      return `Derivation Index`;
    }
  })}</div>`}</div></div>`;
});
const InputWalletConnect = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $walletconnectIsDisconnecting, $$unsubscribe_walletconnectIsDisconnecting;
  let $walletconnectIsConnecting, $$unsubscribe_walletconnectIsConnecting;
  let $walletconnectAccount, $$unsubscribe_walletconnectAccount;
  $$unsubscribe_walletconnectIsDisconnecting = subscribe(walletconnectIsDisconnecting, (value) => $walletconnectIsDisconnecting = value);
  $$unsubscribe_walletconnectIsConnecting = subscribe(walletconnectIsConnecting, (value) => $walletconnectIsConnecting = value);
  $$unsubscribe_walletconnectAccount = subscribe(walletconnectAccount, (value) => $walletconnectAccount = value);
  const raindexClient = useRaindexClient();
  let { priorityChainIds = void 0 } = $$props;
  let { onConnect = () => {
  } } = $$props;
  const networks = raindexClient.getAllNetworks();
  if ($$props.priorityChainIds === void 0 && $$bindings.priorityChainIds && priorityChainIds !== void 0) $$bindings.priorityChainIds(priorityChainIds);
  if ($$props.onConnect === void 0 && $$bindings.onConnect && onConnect !== void 0) $$bindings.onConnect(onConnect);
  $$unsubscribe_walletconnectIsDisconnecting();
  $$unsubscribe_walletconnectIsConnecting();
  $$unsubscribe_walletconnectAccount();
  return `${validate_component(Alert, "Alert").$$render(
    $$result,
    {
      color: "yellow",
      border: true,
      class: "mb-8"
    },
    {},
    {
      icon: () => {
        return `${validate_component(IconWarning, "IconWarning").$$render($$result, { slot: "icon" }, {}, {})}`;
      },
      default: () => {
        return `Only mobile wallets are supported in WalletConnect.`;
      }
    }
  )} ${networks.error ? `${validate_component(Alert, "Alert").$$render(
    $$result,
    {
      color: "red",
      border: true,
      class: "mb-8"
    },
    {},
    {
      icon: () => {
        return `${validate_component(IconWarning, "IconWarning").$$render($$result, { slot: "icon" }, {}, {})}`;
      },
      default: () => {
        return `${escape(networks.error.readableMsg)}`;
      }
    }
  )}` : `<div class="flex w-full justify-end space-x-2">${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      color: "primary",
      class: "w-full px-2 py-1",
      size: "lg",
      pill: true,
      loading: $walletconnectIsDisconnecting || $walletconnectIsConnecting
    },
    {},
    {
      default: () => {
        return `${$walletconnectAccount ? `${validate_component(Hash, "Hash").$$render(
          $$result,
          {
            type: HashType.Wallet,
            value: $walletconnectAccount
          },
          {},
          {}
        )}` : `Connect`}`;
      }
    }
  )} ${$walletconnectAccount ? `${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      color: "red",
      class: "min-w-fit px-2 py-1",
      size: "lg",
      pill: true,
      loading: $walletconnectIsDisconnecting || $walletconnectIsConnecting
    },
    {},
    {
      default: () => {
        return `Disconnect`;
      }
    }
  )}` : ``}</div>`}`;
});
const formatBlockExplorerTransactionUrl = (chainId, hash) => {
  const chain = Object.values(chains).find((chain2) => chain2.id === chainId);
  if (chain?.blockExplorers) {
    return chain.blockExplorers.default.url + `/tx/${hash}`;
  } else {
    return "";
  }
};
function formatEthersTransactionError(e) {
  if (typeof e === "object") {
    return `Transaction failed, error: 
    ${JSON.stringify(e)}`;
  } else if (typeof e === "string") return e;
  else if (e instanceof Error) return e.message;
  else {
    return "Transaction failed!";
  }
}
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      enabled: browser
    }
  }
});
export {
  ClipboardOutline as C,
  ExclamationCircleSolid as E,
  Hash as H,
  IconLedger as I,
  Modal as M,
  Popper as P,
  RAINDEX_CLIENT_CONTEXT_KEY as R,
  Tooltip_1 as T,
  WalletOutline as W,
  IconWalletConnect as a,
  InputLedgerWallet as b,
  InputWalletConnect as c,
  IconWarning as d,
  HashType as e,
  formatBlockExplorerTransactionUrl as f,
  Tooltip as g,
  walletConnectNetwork as h,
  formatEthersTransactionError as i,
  browser as j,
  ledgerWalletDerivationIndex as k,
  ledgerWalletAddress as l,
  Helper as m,
  walletconnectProvider as n,
  queryClient as q,
  useRaindexClient as u,
  walletconnectAccount as w
};
//# sourceMappingURL=queryClient.js.map
