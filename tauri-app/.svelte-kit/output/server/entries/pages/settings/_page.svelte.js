import { c as create_ssr_component, a as compute_rest_props, g as getContext, b as spread, d as escape_object, e as escape_attribute_value, f as add_attribute, k as subscribe, v as validate_component, h as escape } from "../../../chunks/ssr.js";
import { r as reportErrorToSentry, l as SentrySeverityLevel, p as settingsFile, A as Alert, S as Spinner, b as settingsText } from "../../../chunks/sentry.js";
import { C as CodeMirror } from "../../../chunks/CodeMirror.js";
import { a as codeMirrorTheme } from "../../../chunks/darkMode.js";
import { yaml } from "@codemirror/lang-yaml";
import { b as checkSettingsErrors, d as checkConfigErrors, u as useDebouncedFn, F as FileTextarea } from "../../../chunks/asyncDebounce.js";
import { RawRainlangExtension } from "codemirror-rainlang";
import { twMerge } from "tailwind-merge";
import { p as page } from "../../../chunks/stores.js";
import { P as PageHeader } from "../../../chunks/PageHeader.js";
const CheckOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "check outline" } = $$props;
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
      { viewBox: "0 0 16 12" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M1 5.917 5.724 10.5 15 1.5"></path></svg> `;
});
const CloseOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "close outline" } = $$props;
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
      { viewBox: "0 0 14 14" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"></path></svg> `;
});
async function applySettings(settingsContent, settingsTextStore) {
  try {
    await checkSettingsErrors([settingsContent]);
    settingsTextStore.set(settingsContent);
    return { settingsStatus: "success" };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    reportErrorToSentry(error, SentrySeverityLevel.Info);
    return { settingsStatus: "error", errorMessage };
  }
}
const css = {
  code: ".ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]{background-color:inherit}",
  map: `{"version":3,"file":"CodeMirrorConfigSource.svelte","sources":["CodeMirrorConfigSource.svelte"],"sourcesContent":["<script lang=\\"ts\\">import CodeMirror from \\"svelte-codemirror-editor\\";\\nimport { codeMirrorTheme } from \\"$lib/stores/darkMode\\";\\nimport { yaml } from \\"@codemirror/lang-yaml\\";\\nimport { checkConfigErrors } from \\"$lib/services/configCodemirrorProblems\\";\\nimport { RawRainlangExtension } from \\"codemirror-rainlang\\";\\nimport { openLintPanel } from \\"@codemirror/lint\\";\\nexport let value;\\nexport let disabled = false;\\nexport let styles = {};\\nconst configStringExtension = new RawRainlangExtension({\\n  hover: async () => null,\\n  completion: async () => null,\\n  diagnostics: async (textDocument) => checkConfigErrors([textDocument.text])\\n});\\n<\/script>\\n\\n<CodeMirror\\n  bind:value\\n  extensions={[configStringExtension]}\\n  lang={yaml()}\\n  theme={$codeMirrorTheme}\\n  readonly={disabled}\\n  useTab={true}\\n  tabSize={2}\\n  styles={{\\n    '&': {\\n      width: '100%',\\n    },\\n    ...styles,\\n  }}\\n  on:ready={(e) => {\\n    openLintPanel(e.detail);\\n  }}\\n/>\\n\\n<style global>\\n  :global(.ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]) {\\n    background-color: inherit;\\n  }\\n</style>\\n"],"names":[],"mappings":"AAoCU,8CAAgD,CACtD,gBAAgB,CAAE,OACpB"}`
};
const CodeMirrorConfigSource = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $codeMirrorTheme, $$unsubscribe_codeMirrorTheme;
  $$unsubscribe_codeMirrorTheme = subscribe(codeMirrorTheme, (value2) => $codeMirrorTheme = value2);
  let { value } = $$props;
  let { disabled = false } = $$props;
  let { styles = {} } = $$props;
  const configStringExtension = new RawRainlangExtension({
    hover: async () => null,
    completion: async () => null,
    diagnostics: async (textDocument) => checkConfigErrors([textDocument.text])
  });
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  if ($$props.disabled === void 0 && $$bindings.disabled && disabled !== void 0) $$bindings.disabled(disabled);
  if ($$props.styles === void 0 && $$bindings.styles && styles !== void 0) $$bindings.styles(styles);
  $$result.css.add(css);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `${validate_component(CodeMirror, "CodeMirror").$$render(
      $$result,
      {
        extensions: [configStringExtension],
        lang: yaml(),
        theme: $codeMirrorTheme,
        readonly: disabled,
        useTab: true,
        tabSize: 2,
        styles: { "&": { width: "100%" }, ...styles },
        value
      },
      {
        value: ($$value) => {
          value = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  $$unsubscribe_codeMirrorTheme();
  return $$rendered;
});
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $settingsFile, $$unsubscribe_settingsFile;
  let $page, $$unsubscribe_page;
  $$unsubscribe_settingsFile = subscribe(settingsFile, (value) => $settingsFile = value);
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  let settingsStatus = "checking";
  let errorMessage = void 0;
  let height = 500;
  async function handleApply(settingsContent) {
    settingsStatus = "checking";
    errorMessage = void 0;
    const result = await applySettings(settingsContent, settingsText);
    settingsStatus = result.settingsStatus;
    if (result.errorMessage) {
      errorMessage = result.errorMessage;
    }
  }
  const { debouncedFn: debouncedHandleApply } = useDebouncedFn(handleApply, 1e3);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    {
      if ($settingsFile.text !== void 0 && typeof $settingsFile.text === "string" && $settingsFile.text.trim() !== "") {
        debouncedHandleApply($settingsFile.text);
      }
    }
    $$rendered = `<div class="mb-4">${validate_component(PageHeader, "PageHeader").$$render(
      $$result,
      {
        title: "Settings",
        pathname: $page.url.pathname
      },
      {},
      {}
    )} ${validate_component(Alert, "Alert").$$render(
      $$result,
      {
        color: "blue",
        class: "mb-4 mt-8 text-lg"
      },
      {},
      {
        default: () => {
          return `Looking for some settings to get started? Check out the <a class="underline" target="_blank" href="https://docs.rainlang.xyz/raindex/getting-started" data-svelte-h="svelte-dvty0h">getting started guide</a>`;
        }
      }
    )}</div> ${validate_component(FileTextarea, "FileTextarea").$$render($$result, { textFile: settingsFile }, {}, {
      textarea: () => {
        return `${validate_component(CodeMirrorConfigSource, "CodeMirrorConfigSource").$$render(
          $$result,
          {
            styles: {
              "&": {
                maxHeight: `${height - (errorMessage ? 35 : 0)}px`,
                height: "100%"
              }
            },
            value: $settingsFile.text
          },
          {
            value: ($$value) => {
              $settingsFile.text = $$value;
              $$settled = false;
            }
          },
          {}
        )} `;
      },
      alert: () => {
        return `${settingsStatus === "checking" ? `${validate_component(Alert, "Alert").$$render(
          $$result,
          {
            color: "blue",
            class: "flex h-10 items-center text-blue-600 dark:text-blue-400"
          },
          {},
          {
            default: () => {
              return `${validate_component(Spinner, "Spinner").$$render($$result, { class: "mr-2", size: "4" }, {}, {})} <span data-svelte-h="svelte-anso4r">Checking settings...</span>`;
            }
          }
        )}` : `${settingsStatus === "success" ? `${validate_component(Alert, "Alert").$$render(
          $$result,
          {
            color: "green",
            class: "flex h-10 items-center text-green-600 dark:text-green-400"
          },
          {},
          {
            default: () => {
              return `${validate_component(CheckOutline, "CheckOutline").$$render($$result, { class: "mr-2", size: "xs" }, {}, {})} <span data-svelte-h="svelte-l73xql">Settings applied successfully</span>`;
            }
          }
        )}` : `${settingsStatus === "error" ? `${validate_component(Alert, "Alert").$$render(
          $$result,
          {
            color: "red",
            class: "flex flex-col text-red-600 dark:text-red-400"
          },
          {},
          {
            default: () => {
              return `<div class="mb-2 flex items-center">${validate_component(CloseOutline, "CloseOutline").$$render($$result, { class: "mr-2", size: "xs" }, {}, {})} <span data-svelte-h="svelte-um0cv7">Error applying settings</span></div> <span>${escape(errorMessage)}</span>`;
            }
          }
        )}` : ``}`}`}`;
      }
    })}`;
  } while (!$$settled);
  $$unsubscribe_settingsFile();
  $$unsubscribe_page();
  return $$rendered;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
