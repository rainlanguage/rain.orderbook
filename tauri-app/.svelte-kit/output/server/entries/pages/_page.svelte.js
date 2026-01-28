import { c as create_ssr_component, k as subscribe, f as add_attribute, v as validate_component } from "../../chunks/ssr.js";
import { c as colorTheme, B as Button } from "../../chunks/darkMode.js";
import { l as logoDark, a as logoLight, I as IconTelegram } from "../../chunks/logo-dark.js";
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $colorTheme, $$unsubscribe_colorTheme;
  $$unsubscribe_colorTheme = subscribe(colorTheme, (value) => $colorTheme = value);
  $$unsubscribe_colorTheme();
  return `<div class="mx-auto flex max-w-prose flex-col items-center gap-y-8 pt-12"><img data-testid="logo" alt="Raindex logo" class="w-[400px]"${add_attribute("src", $colorTheme === "dark" ? logoDark : logoLight, 0)}> <div data-testid="description" class="mt-4 text-center text-2xl" data-svelte-h="svelte-jyzrmb">Raindex allows anyone to write, deploy and manage token trading orders, written in Rainlang, on
    any EVM network.</div> <div class="flex items-center gap-x-2">${validate_component(Button, "Button").$$render(
    $$result,
    {
      "data-testid": "community-link",
      target: "_blank",
      href: "https://t.me/+W0aQ36ptN_E2MjZk"
    },
    {},
    {
      default: () => {
        return `${validate_component(IconTelegram, "IconTelegram").$$render($$result, {}, {}, {})} <span class="ml-2" data-svelte-h="svelte-1lkwu4t">Join the community</span>`;
      }
    }
  )} ${validate_component(Button, "Button").$$render(
    $$result,
    {
      "data-testid": "get-started-link",
      target: "_blank",
      href: "https://docs.rainlang.xyz/raindex/getting-started"
    },
    {},
    {
      default: () => {
        return `Get started`;
      }
    }
  )}</div> <div style="position: relative; padding-bottom: 64.63195691202873%; height: 0;" class="w-full" data-svelte-h="svelte-170xmln"><iframe data-testid="demo-iframe" title="Raindex Demo" src="https://www.loom.com/embed/fca750f31f0a43258891cea0ddacb588?sid=21583276-742b-49d5-a7db-b6e9e01ca418" frameborder="0" allowfullscreen style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;"></iframe></div></div>`;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
