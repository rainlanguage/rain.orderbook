import { c as create_ssr_component, b as spread, d as escape_object, a as compute_rest_props, g as getContext, e as escape_attribute_value, f as add_attribute, k as subscribe, v as validate_component, h as escape } from "./ssr.js";
import { twMerge } from "tailwind-merge";
import { j as browser } from "./queryClient.js";
import { QueryClient } from "@tanstack/svelte-query";
import { S as Spinner } from "./sentry.js";
import * as chains from "viem/chains";
const CardProperty = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  return `<div${spread([escape_object($$props)], {})}><h5 data-testid="card-property-key" class="text-md mb-1 w-full tracking-tight text-gray-400 dark:text-gray-400">${slots.key ? slots.key({}) : ``}</h5> <p class="font-regular break-all leading-tight text-gray-800 dark:text-white">${slots.value ? slots.value({}) : ``}</p></div>`;
});
const ArrowDownToBracketOutline = create_ssr_component(($$result, $$props, $$bindings, slots) => {
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
  let { ariaLabel = "arrow down to bracket outline" } = $$props;
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
      { viewBox: "0 0 16 18" }
    ],
    {}
  )}><path stroke="currentColor"${add_attribute("stroke-linecap", strokeLinecap, 0)}${add_attribute("stroke-linejoin", strokeLinejoin, 0)}${add_attribute("stroke-width", strokeWidth, 0)} d="M8 1.059v10.425m0 0 4-3.791m-4 3.79-4-3.79m11 3.79v2.844c0 .502-.21.985-.586 1.34a2.058 2.058 0 0 1-1.414.555H3c-.53 0-1.04-.2-1.414-.555A1.847 1.847 0 0 1 1 14.327v-2.843"></path></svg> `;
});
new QueryClient({
  defaultOptions: {
    queries: {
      enabled: browser
    }
  }
});
const invalidateTanstackQueries = async (queryClient, queryKey) => {
  try {
    await queryClient.invalidateQueries({
      queryKey,
      refetchType: "all",
      exact: false
    });
  } catch {
    throw new Error("Failed to refresh data.");
  }
};
const css = {
  code: ".tanstack-detail-grid.svelte-xcv0jb.svelte-xcv0jb{display:flex;flex-direction:column;gap:1rem;width:100%}@media(min-width: 1024px){.tanstack-detail-grid.svelte-xcv0jb.svelte-xcv0jb{flex-direction:row}.tanstack-detail-grid.svelte-xcv0jb>.svelte-xcv0jb:first-child{flex:1}.tanstack-detail-grid.svelte-xcv0jb>.svelte-xcv0jb:last-child{flex:2}}",
  map: '{"version":3,"file":"TanstackPageContentDetail.svelte","sources":["TanstackPageContentDetail.svelte"],"sourcesContent":["<script generics=\\"T\\">import { Spinner } from \\"flowbite-svelte\\";\\nexport let query;\\nexport let emptyMessage = \\"Not found\\";\\nlet data;\\n$: if ($query.data) {\\n  data = $query.data;\\n}\\n<\/script>\\n\\n{#if data}\\n\\t<div class=\\"mb-6 flex items-end justify-between\\">\\n\\t\\t<slot name=\\"top\\" {data} />\\n\\t</div>\\n\\t<div class=\\"tanstack-detail-grid\\">\\n\\t\\t<div class=\\"flex flex-col gap-y-6 lg:col-span-1\\">\\n\\t\\t\\t<slot name=\\"card\\" {data} />\\n\\t\\t</div>\\n\\t\\t<div class=\\"h-[500px] lg:col-span-2\\">\\n\\t\\t\\t<slot name=\\"chart\\" {data} />\\n\\t\\t</div>\\n\\t</div>\\n\\t<div class=\\"w-full\\">\\n\\t\\t<slot name=\\"below\\" {data} />\\n\\t</div>\\n{:else if $query.isFetching || $query.isLoading}\\n\\t<div class=\\"flex h-16 w-full items-center justify-center\\">\\n\\t\\t<Spinner class=\\"h-8 w-8\\" color=\\"white\\" data-testid=\\"loadingSpinner\\" />\\n\\t</div>\\n{:else}\\n\\t<div data-testid=\\"emptyMessage\\" class=\\"text-center text-gray-900 dark:text-white\\">\\n\\t\\t{emptyMessage}\\n\\t</div>\\n{/if}\\n\\n<style>\\n\\t.tanstack-detail-grid {\\n\\t\\tdisplay: flex;\\n\\t\\tflex-direction: column;\\n\\t\\tgap: 1rem;\\n\\t\\twidth: 100%;\\n\\t}\\n\\n\\t@media (min-width: 1024px) {\\n\\t\\t.tanstack-detail-grid {\\n\\t\\t\\tflex-direction: row;\\n\\t\\t}\\n\\n\\t\\t.tanstack-detail-grid > :first-child {\\n\\t\\t\\tflex: 1;\\n\\t\\t}\\n\\n\\t\\t.tanstack-detail-grid > :last-child {\\n\\t\\t\\tflex: 2;\\n\\t\\t}\\n\\t}\\n</style>\\n"],"names":[],"mappings":"AAmCC,iDAAsB,CACrB,OAAO,CAAE,IAAI,CACb,cAAc,CAAE,MAAM,CACtB,GAAG,CAAE,IAAI,CACT,KAAK,CAAE,IACR,CAEA,MAAO,YAAY,MAAM,CAAE,CAC1B,iDAAsB,CACrB,cAAc,CAAE,GACjB,CAEA,mCAAqB,eAAG,YAAa,CACpC,IAAI,CAAE,CACP,CAEA,mCAAqB,eAAG,WAAY,CACnC,IAAI,CAAE,CACP,CACD"}'
};
const TanstackPageContentDetail = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $query, $$unsubscribe_query;
  let { query } = $$props;
  $$unsubscribe_query = subscribe(query, (value) => $query = value);
  let { emptyMessage = "Not found" } = $$props;
  let data;
  if ($$props.query === void 0 && $$bindings.query && query !== void 0) $$bindings.query(query);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  $$result.css.add(css);
  {
    if ($query.data) {
      data = $query.data;
    }
  }
  $$unsubscribe_query();
  return `${data ? `<div class="mb-6 flex items-end justify-between">${slots.top ? slots.top({ data }) : ``}</div> <div class="tanstack-detail-grid svelte-xcv0jb"><div class="flex flex-col gap-y-6 lg:col-span-1 svelte-xcv0jb">${slots.card ? slots.card({ data }) : ``}</div> <div class="h-[500px] lg:col-span-2 svelte-xcv0jb">${slots.chart ? slots.chart({ data }) : ``}</div></div> <div class="w-full">${slots.below ? slots.below({ data }) : ``}</div>` : `${$query.isFetching || $query.isLoading ? `<div class="flex h-16 w-full items-center justify-center">${validate_component(Spinner, "Spinner").$$render(
    $$result,
    {
      class: "h-8 w-8",
      color: "white",
      "data-testid": "loadingSpinner"
    },
    {},
    {}
  )}</div>` : `<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">${escape(emptyMessage)}</div>`}`}`;
});
const getExplorerLink = (hash, chainId, type) => {
  const chain = Object.values(chains).find((chain2) => chain2.id === chainId);
  if (chain?.blockExplorers) {
    return chain.blockExplorers.default.url + `/${type}/${hash}`;
  }
  return "";
};
export {
  ArrowDownToBracketOutline as A,
  CardProperty as C,
  TanstackPageContentDetail as T,
  getExplorerLink as g,
  invalidateTanstackQueries as i
};
//# sourceMappingURL=getExplorerLink.js.map
