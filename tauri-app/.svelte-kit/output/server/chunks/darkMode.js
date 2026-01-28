import { c as create_ssr_component, a as compute_rest_props, g as getContext, b as spread, e as escape_attribute_value, d as escape_object } from "./ssr.js";
import { twMerge } from "tailwind-merge";
import { w as writable, d as derived } from "./index.js";
import { createTheme } from "thememirror";
import { tags } from "@lezer/highlight";
import { ColorType } from "lightweight-charts";
const void_element_names = /^(?:area|base|br|col|command|embed|hr|img|input|keygen|link|meta|param|source|track|wbr)$/;
function is_void(name) {
  return void_element_names.test(name) || name.toLowerCase() === "!doctype";
}
const Button = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["pill", "outline", "size", "href", "type", "color", "shadow", "tag", "checked"]);
  const group = getContext("group");
  let { pill = false } = $$props;
  let { outline = false } = $$props;
  let { size = group ? "sm" : "md" } = $$props;
  let { href = void 0 } = $$props;
  let { type = "button" } = $$props;
  let { color = group ? outline ? "dark" : "alternative" : "primary" } = $$props;
  let { shadow = false } = $$props;
  let { tag = "button" } = $$props;
  let { checked = void 0 } = $$props;
  const colorClasses = {
    alternative: "text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 dark:bg-gray-800 dark:text-gray-400 hover:text-primary-700 focus-within:text-primary-700 dark:focus-within:text-white dark:hover:text-white dark:hover:bg-gray-700",
    blue: "text-white bg-blue-700 hover:bg-blue-800 dark:bg-blue-600 dark:hover:bg-blue-700",
    dark: "text-white bg-gray-800 hover:bg-gray-900 dark:bg-gray-800 dark:hover:bg-gray-700",
    green: "text-white bg-green-700 hover:bg-green-800 dark:bg-green-600 dark:hover:bg-green-700",
    light: "text-gray-900 bg-white border border-gray-300 hover:bg-gray-100 dark:bg-gray-800 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:border-gray-600",
    primary: "text-white bg-primary-700 hover:bg-primary-800 dark:bg-primary-600 dark:hover:bg-primary-700",
    purple: "text-white bg-purple-700 hover:bg-purple-800 dark:bg-purple-600 dark:hover:bg-purple-700",
    red: "text-white bg-red-700 hover:bg-red-800 dark:bg-red-600 dark:hover:bg-red-700",
    yellow: "text-white bg-yellow-400 hover:bg-yellow-500 ",
    none: ""
  };
  const colorCheckedClasses = {
    alternative: "text-primary-700 border dark:text-primary-500 bg-gray-100 dark:bg-gray-700 border-gray-300 shadow-gray-300 dark:shadow-gray-800 shadow-inner",
    blue: "text-blue-900 bg-blue-400 dark:bg-blue-500 shadow-blue-700 dark:shadow-blue-800 shadow-inner",
    dark: "text-white bg-gray-500 dark:bg-gray-600 shadow-gray-800 dark:shadow-gray-900 shadow-inner",
    green: "text-green-900 bg-green-400 dark:bg-green-500 shadow-green-700 dark:shadow-green-800 shadow-inner",
    light: "text-gray-900 bg-gray-100 border border-gray-300 dark:bg-gray-500 dark:text-gray-900 dark:border-gray-700 shadow-gray-300 dark:shadow-gray-700 shadow-inner",
    primary: "text-primary-900 bg-primary-400 dark:bg-primary-500 shadow-primary-700 dark:shadow-primary-800 shadow-inner",
    purple: "text-purple-900 bg-purple-400 dark:bg-purple-500 shadow-purple-700 dark:shadow-purple-800 shadow-inner",
    red: "text-red-900 bg-red-400 dark:bg-red-500 shadow-red-700 dark:shadow-red-800 shadow-inner",
    yellow: "text-yellow-900 bg-yellow-300 dark:bg-yellow-400 shadow-yellow-500 dark:shadow-yellow-700 shadow-inner",
    none: ""
  };
  const coloredFocusClasses = {
    alternative: "focus-within:ring-gray-200 dark:focus-within:ring-gray-700",
    blue: "focus-within:ring-blue-300 dark:focus-within:ring-blue-800",
    dark: "focus-within:ring-gray-300 dark:focus-within:ring-gray-700",
    green: "focus-within:ring-green-300 dark:focus-within:ring-green-800",
    light: "focus-within:ring-gray-200 dark:focus-within:ring-gray-700",
    primary: "focus-within:ring-primary-300 dark:focus-within:ring-primary-800",
    purple: "focus-within:ring-purple-300 dark:focus-within:ring-purple-900",
    red: "focus-within:ring-red-300 dark:focus-within:ring-red-900",
    yellow: "focus-within:ring-yellow-300 dark:focus-within:ring-yellow-900",
    none: ""
  };
  const coloredShadowClasses = {
    alternative: "shadow-gray-500/50 dark:shadow-gray-800/80",
    blue: "shadow-blue-500/50 dark:shadow-blue-800/80",
    dark: "shadow-gray-500/50 dark:shadow-gray-800/80",
    green: "shadow-green-500/50 dark:shadow-green-800/80",
    light: "shadow-gray-500/50 dark:shadow-gray-800/80",
    primary: "shadow-primary-500/50 dark:shadow-primary-800/80",
    purple: "shadow-purple-500/50 dark:shadow-purple-800/80",
    red: "shadow-red-500/50 dark:shadow-red-800/80 ",
    yellow: "shadow-yellow-500/50 dark:shadow-yellow-800/80 ",
    none: ""
  };
  const outlineClasses = {
    alternative: "text-gray-900 dark:text-gray-400 hover:text-white border border-gray-800 hover:bg-gray-900 focus-within:bg-gray-900 focus-within:text-white focus-within:ring-gray-300 dark:border-gray-600 dark:hover:text-white dark:hover:bg-gray-600 dark:focus-within:ring-gray-800",
    blue: "text-blue-700 hover:text-white border border-blue-700 hover:bg-blue-800 dark:border-blue-500 dark:text-blue-500 dark:hover:text-white dark:hover:bg-blue-600",
    dark: "text-gray-900 hover:text-white border border-gray-800 hover:bg-gray-900 focus-within:bg-gray-900 focus-within:text-white dark:border-gray-600 dark:hover:text-white dark:hover:bg-gray-600",
    green: "text-green-700 hover:text-white border border-green-700 hover:bg-green-800 dark:border-green-500 dark:text-green-500 dark:hover:text-white dark:hover:bg-green-600",
    light: "text-gray-500 hover:text-gray-900 bg-white border border-gray-200 dark:border-gray-600 dark:hover:text-white dark:text-gray-400 hover:bg-gray-50 dark:bg-gray-700 dark:hover:bg-gray-600",
    primary: "text-primary-700 hover:text-white border border-primary-700 hover:bg-primary-700 dark:border-primary-500 dark:text-primary-500 dark:hover:text-white dark:hover:bg-primary-600",
    purple: "text-purple-700 hover:text-white border border-purple-700 hover:bg-purple-800 dark:border-purple-400 dark:text-purple-400 dark:hover:text-white dark:hover:bg-purple-500",
    red: "text-red-700 hover:text-white border border-red-700 hover:bg-red-800 dark:border-red-500 dark:text-red-500 dark:hover:text-white dark:hover:bg-red-600",
    yellow: "text-yellow-400 hover:text-white border border-yellow-400 hover:bg-yellow-500 dark:border-yellow-300 dark:text-yellow-300 dark:hover:text-white dark:hover:bg-yellow-400",
    none: ""
  };
  const sizeClasses = {
    xs: "px-3 py-2 text-xs",
    sm: "px-4 py-2 text-sm",
    md: "px-5 py-2.5 text-sm",
    lg: "px-5 py-3 text-base",
    xl: "px-6 py-3.5 text-base"
  };
  const hasBorder = () => outline || color === "alternative" || color === "light";
  let buttonClass;
  if ($$props.pill === void 0 && $$bindings.pill && pill !== void 0) $$bindings.pill(pill);
  if ($$props.outline === void 0 && $$bindings.outline && outline !== void 0) $$bindings.outline(outline);
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.href === void 0 && $$bindings.href && href !== void 0) $$bindings.href(href);
  if ($$props.type === void 0 && $$bindings.type && type !== void 0) $$bindings.type(type);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.shadow === void 0 && $$bindings.shadow && shadow !== void 0) $$bindings.shadow(shadow);
  if ($$props.tag === void 0 && $$bindings.tag && tag !== void 0) $$bindings.tag(tag);
  if ($$props.checked === void 0 && $$bindings.checked && checked !== void 0) $$bindings.checked(checked);
  buttonClass = twMerge(
    "text-center font-medium",
    group ? "focus-within:ring-2" : "focus-within:ring-4",
    group && "focus-within:z-10",
    group || "focus-within:outline-none",
    "inline-flex items-center justify-center " + sizeClasses[size],
    outline && checked && "border dark:border-gray-900",
    outline && checked && colorCheckedClasses[color],
    outline && !checked && outlineClasses[color],
    !outline && checked && colorCheckedClasses[color],
    !outline && !checked && colorClasses[color],
    color === "alternative" && (group && !checked ? "dark:bg-gray-700 dark:text-white dark:border-gray-700 dark:hover:border-gray-600 dark:hover:bg-gray-600" : "dark:bg-transparent dark:border-gray-600 dark:hover:border-gray-600"),
    outline && color === "dark" && (group ? checked ? "bg-gray-900 border-gray-800 dark:border-white dark:bg-gray-600" : "dark:text-white border-gray-800 dark:border-white" : "dark:text-gray-400 dark:border-gray-700"),
    coloredFocusClasses[color],
    hasBorder() && group && "border-s-0 first:border-s",
    group ? pill && "first:rounded-s-full last:rounded-e-full" || "first:rounded-s-lg last:rounded-e-lg" : pill && "rounded-full" || "rounded-lg",
    shadow && "shadow-lg",
    shadow && coloredShadowClasses[color],
    $$props.disabled && "cursor-not-allowed opacity-50",
    $$props.class
  );
  return `${href ? `<a${spread(
    [
      { href: escape_attribute_value(href) },
      escape_object($$restProps),
      {
        class: escape_attribute_value(buttonClass)
      },
      { role: "button" }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</a>` : `${tag === "button" ? `<button${spread(
    [
      { type: escape_attribute_value(type) },
      escape_object($$restProps),
      {
        class: escape_attribute_value(buttonClass)
      }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</button>` : `${((tag$1) => {
    return tag$1 ? `<${tag}${spread(
      [
        escape_object($$restProps),
        {
          class: escape_attribute_value(buttonClass)
        }
      ],
      {}
    )}>${is_void(tag$1) ? "" : `${slots.default ? slots.default({}) : ``}`}${is_void(tag$1) ? "" : `</${tag$1}>`}` : "";
  })(tag)}`}`} `;
});
const lightCodeMirrorTheme = createTheme({
  variant: "light",
  settings: {
    background: "#ffffff",
    foreground: "#001080",
    caret: "#000000",
    selection: "#add6ff",
    lineHighlight: "#77777740",
    gutterBackground: "#eeeeee",
    gutterForeground: "#237893"
  },
  styles: [
    { tag: tags.comment, color: "#008001" },
    { tag: tags.variableName, color: "#0070c1" },
    { tag: [tags.string, tags.special(tags.brace)], color: "#b55b00" },
    { tag: tags.number, color: "#00b97b" },
    { tag: tags.bool, color: "#0000ff" },
    { tag: tags.null, color: "#0000ff" },
    { tag: tags.unit, color: "#0000ff" },
    { tag: tags.keyword, color: "#af01db" },
    { tag: tags.operator, color: "#000000" },
    { tag: tags.className, color: "#257f99" },
    { tag: tags.meta, color: "#0950a9" },
    { tag: tags.definition(tags.typeName), color: "#257f99" },
    { tag: tags.angleBracket, color: "#213CF1" },
    { tag: tags.brace, color: "#213CF1" },
    { tag: tags.bracket, color: "#213CF1" },
    { tag: tags.squareBracket, color: "#213CF1" },
    { tag: tags.paren, color: "#213CF1" },
    { tag: tags.punctuation, color: "#464646" },
    { tag: tags.separator, color: "#464646" },
    { tag: tags.typeName, color: "#257f99" },
    { tag: tags.tagName, color: "#800000" },
    { tag: tags.attributeName, color: "#eb3d36" },
    { tag: tags.attributeValue, color: "#444444" },
    { tag: tags.content, color: "#b55b00" },
    { tag: [tags.propertyName, tags.definition(tags.propertyName)], color: "#0469ff" },
    { tag: tags.labelName, color: "#4fc4ff" },
    { tag: tags.deleted, color: "#cc0000" }
  ]
});
const darkCodeMirrorTheme = createTheme({
  variant: "dark",
  settings: {
    background: "#1e1e1e",
    foreground: "#d4d4d4",
    caret: "#d4d4d4",
    selection: "#ffffff",
    lineHighlight: "#99999940",
    gutterBackground: "#282828",
    gutterForeground: "#858585"
  },
  styles: [
    { tag: [tags.comment, tags.lineComment], color: "#6c9e57" },
    { tag: tags.variableName, color: "#9cdcfe" },
    { tag: [tags.string, tags.special(tags.brace)], color: "#ce9178" },
    { tag: tags.number, color: "#B6CFA9" },
    { tag: tags.bool, color: "#4fc4ff" },
    { tag: tags.null, color: "#4fc4ff" },
    { tag: tags.unit, color: "#608FC2" },
    { tag: tags.keyword, color: "#d18dcc" },
    { tag: tags.operator, color: "#d4d4d4" },
    { tag: tags.className, color: "#4dcab1" },
    { tag: tags.meta, color: "#608FC2" },
    { tag: tags.definition(tags.typeName), color: "#4fcfb5" },
    { tag: tags.angleBracket, color: "#F9D849" },
    { tag: tags.brace, color: "#F9D849" },
    { tag: tags.bracket, color: "#F9D849" },
    { tag: tags.squareBracket, color: "#F9D849" },
    { tag: tags.paren, color: "#F9D849" },
    { tag: tags.punctuation, color: "#d4d4d4" },
    { tag: tags.separator, color: "#d4d4d4" },
    { tag: tags.typeName, color: "#4ecdb4" },
    { tag: tags.tagName, color: "#59a3df" },
    { tag: tags.attributeName, color: "#ffffff" },
    { tag: tags.attributeValue, color: "#ffffff" },
    { tag: tags.content, color: "#ce9178" },
    { tag: [tags.propertyName, tags.definition(tags.propertyName)], color: "#608FC2" },
    { tag: tags.labelName, color: "#4fc4ff" },
    { tag: tags.deleted, color: "#c86464" }
  ]
});
const darkChartTheme = {
  layout: {
    textColor: "white",
    background: { type: ColorType.Solid, color: "transparent" }
  },
  grid: {
    vertLines: { color: "#484848" },
    horzLines: { color: "#484848" }
  }
};
const lightChartTheme = {
  layout: {
    textColor: "black",
    background: { type: ColorType.Solid, color: "transparent" }
  },
  grid: {
    vertLines: { color: "#ECECEC" },
    horzLines: { color: "#ECECEC" }
  }
};
function readColorTheme() {
  const saved = localStorage.getItem("color-theme");
  if (saved) {
    return saved;
  } else if (document.body.classList.contains("dark")) {
    return "dark";
  } else {
    return "light";
  }
}
const colorTheme = writable(readColorTheme());
colorTheme.subscribe((val) => localStorage.setItem("color-theme", val));
const codeMirrorTheme = derived(
  colorTheme,
  ($colorTheme) => $colorTheme === "dark" ? darkCodeMirrorTheme : lightCodeMirrorTheme
);
const lightweightChartsTheme = derived(
  colorTheme,
  ($colorTheme) => $colorTheme === "dark" ? darkChartTheme : lightChartTheme
);
export {
  Button as B,
  codeMirrorTheme as a,
  colorTheme as c,
  is_void as i,
  lightweightChartsTheme as l
};
//# sourceMappingURL=darkMode.js.map
