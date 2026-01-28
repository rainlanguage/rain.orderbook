import { c as create_ssr_component, a as compute_rest_props, b as spread, e as escape_attribute_value, d as escape_object, f as add_attribute, i as compute_slots, v as validate_component, j as each, h as escape } from "./ssr.js";
import { twMerge } from "tailwind-merge";
const Breadcrumb = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["solid", "navClass", "solidClass", "olClass", "ariaLabel"]);
  let { solid = false } = $$props;
  let { navClass = "flex" } = $$props;
  let { solidClass = "flex px-5 py-3 text-gray-700 border border-gray-200 rounded-lg bg-gray-50 dark:bg-gray-800 dark:border-gray-700" } = $$props;
  let { olClass = "inline-flex items-center space-x-1 rtl:space-x-reverse md:space-x-3 rtl:space-x-reverse" } = $$props;
  let { ariaLabel = "Breadcrumb" } = $$props;
  let classNav = solid ? solidClass : navClass;
  if ($$props.solid === void 0 && $$bindings.solid && solid !== void 0) $$bindings.solid(solid);
  if ($$props.navClass === void 0 && $$bindings.navClass && navClass !== void 0) $$bindings.navClass(navClass);
  if ($$props.solidClass === void 0 && $$bindings.solidClass && solidClass !== void 0) $$bindings.solidClass(solidClass);
  if ($$props.olClass === void 0 && $$bindings.olClass && olClass !== void 0) $$bindings.olClass(olClass);
  if ($$props.ariaLabel === void 0 && $$bindings.ariaLabel && ariaLabel !== void 0) $$bindings.ariaLabel(ariaLabel);
  return `<nav${spread(
    [
      {
        "aria-label": escape_attribute_value(ariaLabel)
      },
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(classNav, $$props.class))
      }
    ],
    {}
  )}><ol${add_attribute("class", twMerge(olClass, $$props.classOl), 0)}>${slots.default ? slots.default({}) : ``}</ol></nav> `;
});
const BreadcrumbItem = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["home", "href", "linkClass", "spanClass", "homeClass"]);
  let $$slots = compute_slots(slots);
  let { home = false } = $$props;
  let { href = void 0 } = $$props;
  let { linkClass = "ms-1 text-sm font-medium text-gray-700 hover:text-gray-900 md:ms-2 dark:text-gray-400 dark:hover:text-white" } = $$props;
  let { spanClass = "ms-1 text-sm font-medium text-gray-500 md:ms-2 dark:text-gray-400" } = $$props;
  let { homeClass = "inline-flex items-center text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white" } = $$props;
  if ($$props.home === void 0 && $$bindings.home && home !== void 0) $$bindings.home(home);
  if ($$props.href === void 0 && $$bindings.href && href !== void 0) $$bindings.href(href);
  if ($$props.linkClass === void 0 && $$bindings.linkClass && linkClass !== void 0) $$bindings.linkClass(linkClass);
  if ($$props.spanClass === void 0 && $$bindings.spanClass && spanClass !== void 0) $$bindings.spanClass(spanClass);
  if ($$props.homeClass === void 0 && $$bindings.homeClass && homeClass !== void 0) $$bindings.homeClass(homeClass);
  return `<li${spread(
    [
      {
        class: escape_attribute_value(twMerge("inline-flex items-center", $$props.class))
      },
      escape_object($$restProps)
    ],
    {}
  )}>${home ? `<a${add_attribute("class", twMerge(homeClass, $$props.classHome), 0)}${add_attribute("href", href, 0)}>${$$slots.icon ? `${slots.icon ? slots.icon({}) : ``}` : `<svg class="w-4 h-4 me-2" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z"></path></svg>`} ${slots.default ? slots.default({}) : ``}</a>` : `${$$slots.icon ? `${slots.icon ? slots.icon({}) : ``}` : `<svg class="w-6 h-6 text-gray-400 rtl:-scale-x-100" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>`} ${href ? `<a${add_attribute("class", twMerge(linkClass, $$props.classLink), 0)}${add_attribute("href", href, 0)}>${slots.default ? slots.default({}) : ``}</a>` : `<span${add_attribute("class", twMerge(spanClass, $$props.classSpan), 0)}>${slots.default ? slots.default({}) : ``}</span>`}`}</li> `;
});
function generateBreadcrumbs(path) {
  const crumbs = path.split("/");
  return crumbs.map((c, i) => ({
    label: c,
    href: crumbs.slice(0, i + 1).join("/")
  })).slice(1, -1);
}
const PageHeader = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let breadcrumbs;
  let { title } = $$props;
  let { pathname } = $$props;
  if ($$props.title === void 0 && $$bindings.title && title !== void 0) $$bindings.title(title);
  if ($$props.pathname === void 0 && $$bindings.pathname && pathname !== void 0) $$bindings.pathname(pathname);
  breadcrumbs = generateBreadcrumbs(pathname);
  return `<div class="mb-4 flex w-full items-center justify-between" data-testid="page-header">${validate_component(Breadcrumb, "Breadcrumb").$$render(
    $$result,
    {
      olClass: "inline-flex items-center rtl:space-x-reverse",
      "aria-label": "Default breadcrumb example"
    },
    {},
    {
      default: () => {
        return `${validate_component(BreadcrumbItem, "BreadcrumbItem").$$render($$result, { href: "/", home: true }, {}, {})} ${each(breadcrumbs, (crumb) => {
          return `${validate_component(BreadcrumbItem, "BreadcrumbItem").$$render(
            $$result,
            {
              href: crumb.href,
              linkClass: "mx-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white first-letter:uppercase"
            },
            {},
            {
              default: () => {
                return `${escape(crumb.label)}`;
              }
            }
          )}`;
        })} ${validate_component(BreadcrumbItem, "BreadcrumbItem").$$render(
          $$result,
          {
            spanClass: "mx-2 text-sm font-medium text-gray-700 dark:text-gray-300 first-letter:uppercase"
          },
          {},
          {
            default: () => {
              return `<span data-testid="breadcrumb-page-title">${escape(title)}</span>`;
            }
          }
        )}`;
      }
    }
  )} <div class="flex flex-col items-end gap-2 lg:flex-row lg:items-center">${slots.warning ? slots.warning({}) : ``} ${slots.actions ? slots.actions({}) : ``}</div></div>`;
});
export {
  PageHeader as P
};
//# sourceMappingURL=PageHeader.js.map
