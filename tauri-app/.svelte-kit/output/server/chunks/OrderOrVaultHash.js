import { c as create_ssr_component, f as add_attribute, v as validate_component } from "./ssr.js";
import { B as Button } from "./darkMode.js";
import { H as Hash, e as HashType } from "./queryClient.js";
import "@fast-check/vitest";
function isOrder(obj) {
  return obj && "orderHash" in obj;
}
function constructHashLink(orderOrVault, type, chainId, orderbookAddress) {
  if (!orderOrVault) {
    return `/${type}`;
  }
  const slug = isOrder(orderOrVault) ? orderOrVault.orderHash : orderOrVault.id;
  return `/${type}/${chainId}-${orderbookAddress}-${slug}`;
}
function isOrderOrVaultActive(orderOrVault) {
  const _isOrder = isOrder(orderOrVault);
  return _isOrder ? orderOrVault.active : false;
}
function extractHash(orderOrVault) {
  const _isOrder = isOrder(orderOrVault);
  return _isOrder ? orderOrVault.orderHash : orderOrVault?.id || "";
}
const OrderOrVaultHash = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let hash;
  let isActive;
  let linkPath;
  let { orderOrVault } = $$props;
  let { type } = $$props;
  let { chainId } = $$props;
  let { orderbookAddress } = $$props;
  if ($$props.orderOrVault === void 0 && $$bindings.orderOrVault && orderOrVault !== void 0) $$bindings.orderOrVault(orderOrVault);
  if ($$props.type === void 0 && $$bindings.type && type !== void 0) $$bindings.type(type);
  if ($$props.chainId === void 0 && $$bindings.chainId && chainId !== void 0) $$bindings.chainId(chainId);
  if ($$props.orderbookAddress === void 0 && $$bindings.orderbookAddress && orderbookAddress !== void 0) $$bindings.orderbookAddress(orderbookAddress);
  hash = extractHash(orderOrVault);
  isActive = isOrderOrVaultActive(orderOrVault);
  linkPath = constructHashLink(orderOrVault, type, chainId, orderbookAddress);
  return `<a data-testid="order-or-vault-hash"${add_attribute("href", linkPath, 0)}>${validate_component(Button, "Button").$$render(
    $$result,
    {
      class: "mr-1 mt-1 px-2 py-1 text-sm",
      color: isActive ? "green" : "yellow",
      "data-testid": "vault-order-input",
      "data-id": hash
    },
    {},
    {
      default: () => {
        return `${validate_component(Hash, "Hash").$$render(
          $$result,
          {
            type: HashType.Identifier,
            value: hash,
            copyOnClick: false
          },
          {},
          {}
        )}`;
      }
    }
  )}</a>`;
});
export {
  OrderOrVaultHash as O
};
//# sourceMappingURL=OrderOrVaultHash.js.map
