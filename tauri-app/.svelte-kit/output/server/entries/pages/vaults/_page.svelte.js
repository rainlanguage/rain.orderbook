import { c as create_ssr_component, n as getAllContexts, k as subscribe, v as validate_component, h as escape, j as each, o as noop } from "../../../chunks/ssr.js";
import { p as page } from "../../../chunks/stores.js";
import { m as Float, o as orderHash, d as activeAccountsItems, c as selectedChainIds, e as showInactiveOrders, h as hideZeroBalanceVaults, f as hideInactiveOrdersVaults, g as activeTokens, i as activeOrderbookAddresses } from "../../../chunks/sentry.js";
import { u as useAccount, Q as QKEY_TOKENS, k as QKEY_VAULTS, D as DEFAULT_PAGE_SIZE, b as DEFAULT_REFRESH_INTERVAL, T as TanstackAppTable, i as handleDepositModal, j as handleWithdrawModal } from "../../../chunks/modal.js";
import { w as writable } from "../../../chunks/index.js";
import { P as PageHeader } from "../../../chunks/PageHeader.js";
import { toHex } from "viem";
import { u as useRaindexClient, T as Tooltip_1, H as Hash, e as HashType } from "../../../chunks/queryClient.js";
import { B as Button } from "../../../chunks/darkMode.js";
import { D as Dropdown } from "../../../chunks/ChevronDownSolid.js";
import { L as ListViewOrderbookFilters, C as Checkbox, D as DotsVerticalOutline, a as DropdownItem } from "../../../chunks/ListViewOrderbookFilters.js";
import { T as TableBodyCell, g as getNetworkName, a as TableHeadCell } from "../../../chunks/order.js";
import { A as ArrowUpFromBracketOutline } from "../../../chunks/ArrowUpFromBracketOutline.js";
import { createQuery, createInfiniteQuery } from "@tanstack/svelte-query";
import { O as OrderOrVaultHash } from "../../../chunks/OrderOrVaultHash.js";
import { u as useToasts } from "../../../chunks/useToasts.js";
const VaultsListTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let owners;
  let tokensQuery;
  let selectedTokens;
  let orderbooksMap;
  let availableOrderbookAddresses;
  let selectedOrderbookAddresses;
  let query;
  let $query, $$unsubscribe_query = noop, $$subscribe_query = () => ($$unsubscribe_query(), $$unsubscribe_query = subscribe(query, ($$value) => $query = $$value), query);
  let $account, $$unsubscribe_account;
  let $hideInactiveOrdersVaults, $$unsubscribe_hideInactiveOrdersVaults;
  let $hideZeroBalanceVaults, $$unsubscribe_hideZeroBalanceVaults;
  let $selectedChainIds, $$unsubscribe_selectedChainIds;
  let $activeOrderbookAddresses, $$unsubscribe_activeOrderbookAddresses;
  let $tokensQuery, $$unsubscribe_tokensQuery = noop, $$subscribe_tokensQuery = () => ($$unsubscribe_tokensQuery(), $$unsubscribe_tokensQuery = subscribe(tokensQuery, ($$value) => $tokensQuery = $$value), tokensQuery);
  let $activeTokens, $$unsubscribe_activeTokens;
  let $showMyItemsOnly, $$unsubscribe_showMyItemsOnly;
  let $activeAccountsItems, $$unsubscribe_activeAccountsItems;
  getAllContexts();
  useToasts();
  let { activeAccountsItems: activeAccountsItems2 } = $$props;
  $$unsubscribe_activeAccountsItems = subscribe(activeAccountsItems2, (value) => $activeAccountsItems = value);
  let { orderHash: orderHash2 } = $$props;
  let { showInactiveOrders: showInactiveOrders2 } = $$props;
  let { hideZeroBalanceVaults: hideZeroBalanceVaults2 } = $$props;
  $$unsubscribe_hideZeroBalanceVaults = subscribe(hideZeroBalanceVaults2, (value) => $hideZeroBalanceVaults = value);
  let { hideInactiveOrdersVaults: hideInactiveOrdersVaults2 } = $$props;
  $$unsubscribe_hideInactiveOrdersVaults = subscribe(hideInactiveOrdersVaults2, (value) => $hideInactiveOrdersVaults = value);
  let { activeTokens: activeTokens2 } = $$props;
  $$unsubscribe_activeTokens = subscribe(activeTokens2, (value) => $activeTokens = value);
  let { selectedChainIds: selectedChainIds2 } = $$props;
  $$unsubscribe_selectedChainIds = subscribe(selectedChainIds2, (value) => $selectedChainIds = value);
  let { showMyItemsOnly } = $$props;
  $$unsubscribe_showMyItemsOnly = subscribe(showMyItemsOnly, (value) => $showMyItemsOnly = value);
  let { activeOrderbookAddresses: activeOrderbookAddresses2 } = $$props;
  $$unsubscribe_activeOrderbookAddresses = subscribe(activeOrderbookAddresses2, (value) => $activeOrderbookAddresses = value);
  let { handleDepositModal: handleDepositModal2 = void 0 } = $$props;
  let { handleWithdrawModal: handleWithdrawModal2 = void 0 } = $$props;
  let { onWithdrawAll = void 0 } = $$props;
  const { account } = useAccount();
  $$unsubscribe_account = subscribe(account, (value) => $account = value);
  const raindexClient = useRaindexClient();
  let selectedVaults = /* @__PURE__ */ new Set();
  let selectedVaultsOnChainId = null;
  const ZERO_FLOAT = Float.parse("0").value;
  const isZeroBalance = (item) => {
    if (!ZERO_FLOAT) return true;
    return item.balance.eq(ZERO_FLOAT).value;
  };
  const isSameChainId = (item, chainId) => {
    return chainId === null || chainId === item.chainId;
  };
  const isDisabled = (item, chainId) => {
    return !isSameChainId(item, chainId) || isZeroBalance(item);
  };
  const AppTable = TanstackAppTable;
  if ($$props.activeAccountsItems === void 0 && $$bindings.activeAccountsItems && activeAccountsItems2 !== void 0) $$bindings.activeAccountsItems(activeAccountsItems2);
  if ($$props.orderHash === void 0 && $$bindings.orderHash && orderHash2 !== void 0) $$bindings.orderHash(orderHash2);
  if ($$props.showInactiveOrders === void 0 && $$bindings.showInactiveOrders && showInactiveOrders2 !== void 0) $$bindings.showInactiveOrders(showInactiveOrders2);
  if ($$props.hideZeroBalanceVaults === void 0 && $$bindings.hideZeroBalanceVaults && hideZeroBalanceVaults2 !== void 0) $$bindings.hideZeroBalanceVaults(hideZeroBalanceVaults2);
  if ($$props.hideInactiveOrdersVaults === void 0 && $$bindings.hideInactiveOrdersVaults && hideInactiveOrdersVaults2 !== void 0) $$bindings.hideInactiveOrdersVaults(hideInactiveOrdersVaults2);
  if ($$props.activeTokens === void 0 && $$bindings.activeTokens && activeTokens2 !== void 0) $$bindings.activeTokens(activeTokens2);
  if ($$props.selectedChainIds === void 0 && $$bindings.selectedChainIds && selectedChainIds2 !== void 0) $$bindings.selectedChainIds(selectedChainIds2);
  if ($$props.showMyItemsOnly === void 0 && $$bindings.showMyItemsOnly && showMyItemsOnly !== void 0) $$bindings.showMyItemsOnly(showMyItemsOnly);
  if ($$props.activeOrderbookAddresses === void 0 && $$bindings.activeOrderbookAddresses && activeOrderbookAddresses2 !== void 0) $$bindings.activeOrderbookAddresses(activeOrderbookAddresses2);
  if ($$props.handleDepositModal === void 0 && $$bindings.handleDepositModal && handleDepositModal2 !== void 0) $$bindings.handleDepositModal(handleDepositModal2);
  if ($$props.handleWithdrawModal === void 0 && $$bindings.handleWithdrawModal && handleWithdrawModal2 !== void 0) $$bindings.handleWithdrawModal(handleWithdrawModal2);
  if ($$props.onWithdrawAll === void 0 && $$bindings.onWithdrawAll && onWithdrawAll !== void 0) $$bindings.onWithdrawAll(onWithdrawAll);
  owners = $activeAccountsItems && Object.values($activeAccountsItems).length > 0 ? Object.values($activeAccountsItems) : $showMyItemsOnly && $account ? [$account] : [];
  $$subscribe_tokensQuery(tokensQuery = createQuery({
    queryKey: [QKEY_TOKENS, $selectedChainIds],
    queryFn: async () => {
      const result = await raindexClient.getAllVaultTokens($selectedChainIds);
      if (result.error) throw new Error(result.error.readableMsg);
      return result.value;
    },
    enabled: true
  }));
  selectedTokens = $activeTokens?.filter((address) => !$tokensQuery.data || $tokensQuery.data.some((t) => t.address === address)) ?? [];
  orderbooksMap = raindexClient.getAllOrderbooks()?.value ?? /* @__PURE__ */ new Map();
  availableOrderbookAddresses = (() => {
    const addrs = [];
    orderbooksMap.forEach((cfg) => {
      if ($selectedChainIds.length === 0 || $selectedChainIds.includes(cfg.network.chainId)) {
        addrs.push(cfg.address.toLowerCase());
      }
    });
    return addrs;
  })();
  selectedOrderbookAddresses = $activeOrderbookAddresses?.filter((address) => availableOrderbookAddresses.includes(address.toLowerCase())) ?? [];
  $$subscribe_query(query = createInfiniteQuery({
    queryKey: [
      QKEY_VAULTS,
      $hideZeroBalanceVaults,
      $hideInactiveOrdersVaults,
      $selectedChainIds,
      owners,
      selectedTokens,
      selectedOrderbookAddresses
    ],
    queryFn: async ({ pageParam }) => {
      const result = await raindexClient.getVaults(
        $selectedChainIds,
        {
          owners,
          hideZeroBalance: $hideZeroBalanceVaults,
          tokens: selectedTokens,
          orderbookAddresses: selectedOrderbookAddresses.length > 0 ? selectedOrderbookAddresses : void 0,
          onlyActiveOrders: $hideInactiveOrdersVaults
        },
        pageParam + 1
      );
      if (result.error) throw new Error(result.error.readableMsg);
      return result.value;
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.items.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : void 0;
    },
    refetchInterval: DEFAULT_REFRESH_INTERVAL,
    enabled: true
  }));
  {
    if (selectedVaults.size > 0 && !$account) {
      selectedVaults = /* @__PURE__ */ new Set();
      selectedVaultsOnChainId = null;
    }
  }
  $$unsubscribe_query();
  $$unsubscribe_account();
  $$unsubscribe_hideInactiveOrdersVaults();
  $$unsubscribe_hideZeroBalanceVaults();
  $$unsubscribe_selectedChainIds();
  $$unsubscribe_activeOrderbookAddresses();
  $$unsubscribe_tokensQuery();
  $$unsubscribe_activeTokens();
  $$unsubscribe_showMyItemsOnly();
  $$unsubscribe_activeAccountsItems();
  return `${$query ? `${validate_component(ListViewOrderbookFilters, "ListViewOrderbookFilters").$$render(
    $$result,
    {
      selectedChainIds: selectedChainIds2,
      activeAccountsItems: activeAccountsItems2,
      showMyItemsOnly,
      showInactiveOrders: showInactiveOrders2,
      orderHash: orderHash2,
      hideZeroBalanceVaults: hideZeroBalanceVaults2,
      hideInactiveOrdersVaults: hideInactiveOrdersVaults2,
      activeTokens: activeTokens2,
      tokensQuery,
      selectedTokens,
      activeOrderbookAddresses: activeOrderbookAddresses2,
      selectedOrderbookAddresses
    },
    {},
    {}
  )} ${validate_component(AppTable, "AppTable").$$render(
    $$result,
    {
      query,
      dataSelector: (page2) => page2.items,
      queryKey: QKEY_VAULTS,
      emptyMessage: "No Vaults Found"
    },
    {},
    {
      bodyRow: ({ item }) => {
        return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "px-0" }, {}, {
          default: () => {
            return `${validate_component(Checkbox, "Checkbox").$$render(
              $$result,
              {
                "data-testid": "vault-checkbox",
                class: `block px-2 py-4 ${$account?.toLowerCase() !== item.owner.toLowerCase() ? "invisible" : ""}`,
                checked: selectedVaults.has(item.id),
                disabled: isDisabled(item, selectedVaultsOnChainId),
                "aria-label": `Select vault ${item.id}`
              },
              {},
              {}
            )} ${$account?.toLowerCase() === item.owner.toLowerCase() && isDisabled(item, selectedVaultsOnChainId) ? `${validate_component(Tooltip_1, "Tooltip").$$render($$result, {}, {}, {
              default: () => {
                return `${escape(isZeroBalance(item) ? "This vault has a zero balance" : "This vault is on a different network")}`;
              }
            })}` : ``}`;
          }
        })} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "px-4 py-2",
            "data-testid": "vault-network"
          },
          {},
          {
            default: () => {
              return `${escape(getNetworkName(Number(item.chainId)))}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all px-4 py-4",
            "data-testid": "vault-id"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Identifier,
                  value: toHex(item.vaultId)
                },
                {},
                {}
              )}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all px-4 py-2 min-w-48",
            "data-testid": "vault-orderbook"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Identifier,
                  value: item.orderbook
                },
                {},
                {}
              )}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all px-4 py-2 min-w-48",
            "data-testid": "vault-owner"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render($$result, { type: HashType.Wallet, value: item.owner }, {}, {})}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-word p-2 min-w-48",
            "data-testid": "vault-token"
          },
          {},
          {
            default: () => {
              return `${escape(item.token.name)}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all p-2 min-w-48",
            "data-testid": "vault-balance"
          },
          {},
          {
            default: () => {
              return `${escape(`${item.formattedBalance} ${item.token.symbol}`)}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all p-2 min-w-48" }, {}, {
          default: () => {
            return `${item.ordersAsInput.length > 0 ? `<div data-testid="vault-order-inputs" class="flex flex-wrap items-end justify-start">${each(item.ordersAsInput.slice(0, 3), (order) => {
              return `${validate_component(OrderOrVaultHash, "OrderOrVaultHash").$$render(
                $$result,
                {
                  type: "orders",
                  orderOrVault: order,
                  chainId: item.chainId,
                  orderbookAddress: item.orderbook
                },
                {},
                {}
              )}`;
            })} ${item.ordersAsInput.length > 3 ? `...` : ``}</div>` : ``}`;
          }
        })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "break-all p-2 min-w-48" }, {}, {
          default: () => {
            return `${item.ordersAsOutput.length > 0 ? `<div data-testid="vault-order-outputs" class="flex flex-wrap items-end justify-start">${each(item.ordersAsOutput.slice(0, 3), (order) => {
              return `${validate_component(OrderOrVaultHash, "OrderOrVaultHash").$$render(
                $$result,
                {
                  type: "orders",
                  orderOrVault: order,
                  chainId: item.chainId,
                  orderbookAddress: item.orderbook
                },
                {},
                {}
              )}`;
            })} ${item.ordersAsOutput.length > 3 ? `...` : ``}</div>` : ``}`;
          }
        })} ${handleDepositModal2 && handleWithdrawModal2 && item.owner.toLowerCase() === $account?.toLowerCase() ? `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "px-0 text-right" }, {}, {
          default: () => {
            return `${validate_component(Button, "Button").$$render(
              $$result,
              {
                color: "alternative",
                outline: false,
                "data-testid": "vault-menu",
                id: `vault-menu-${item.id}`,
                class: "mr-2 border-none px-2"
              },
              {},
              {
                default: () => {
                  return `${validate_component(DotsVerticalOutline, "DotsVerticalOutline").$$render($$result, { class: "dark:text-white" }, {}, {})}`;
                }
              }
            )}`;
          }
        })} ${validate_component(Dropdown, "Dropdown").$$render(
          $$result,
          {
            "data-testid": "dropdown",
            placement: "bottom-end",
            triggeredBy: `#vault-menu-${item.id}`
          },
          {},
          {
            default: () => {
              return `${validate_component(DropdownItem, "DropdownItem").$$render($$result, { "data-testid": "deposit-button" }, {}, {
                default: () => {
                  return `Deposit`;
                }
              })} ${validate_component(DropdownItem, "DropdownItem").$$render($$result, { "data-testid": "withdraw-button" }, {}, {
                default: () => {
                  return `Withdraw`;
                }
              })}`;
            }
          }
        )}` : ``} `;
      },
      head: () => {
        return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
          default: () => {
            return `<span class="sr-only" data-svelte-h="svelte-9lrab7">Select</span>`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-4" }, {}, {
          default: () => {
            return `Network`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-4 py-4" }, {}, {
          default: () => {
            return `Vault ID`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-4 py-4" }, {}, {
          default: () => {
            return `Orderbook`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-4 py-4" }, {}, {
          default: () => {
            return `Owner`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-2 py-4" }, {}, {
          default: () => {
            return `Token`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-2 py-4" }, {}, {
          default: () => {
            return `Balance`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-3 py-4" }, {}, {
          default: () => {
            return `Input For`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "px-3 py-4" }, {}, {
          default: () => {
            return `Output For`;
          }
        })} `;
      },
      title: () => {
        return `<div class="mt-2 flex w-full justify-between"><div class="flex items-center gap-x-6"><div class="text-3xl font-medium dark:text-white" data-svelte-h="svelte-qv0rgb">Vaults</div> ${validate_component(Button, "Button").$$render(
          $$result,
          {
            size: "xs",
            disabled: !onWithdrawAll || selectedVaults.size === 0,
            "data-testid": "withdraw-all-button"
          },
          {},
          {
            default: () => {
              return `${validate_component(ArrowUpFromBracketOutline, "ArrowUpFromBracketOutline").$$render($$result, { size: "xs", class: "mr-2" }, {}, {})} ${escape(selectedVaults.size > 0 ? `Withdraw selected (${selectedVaults.size})` : "Withdraw vaults")}`;
            }
          }
        )}</div></div>`;
      }
    }
  )}` : ``}`;
});
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  $$unsubscribe_page();
  return `${validate_component(PageHeader, "PageHeader").$$render(
    $$result,
    {
      title: "Vaults",
      pathname: $page.url.pathname
    },
    {},
    {}
  )} ${validate_component(VaultsListTable, "VaultsListTable").$$render(
    $$result,
    {
      orderHash,
      activeAccountsItems,
      selectedChainIds,
      showInactiveOrders,
      hideZeroBalanceVaults,
      hideInactiveOrdersVaults,
      handleDepositModal,
      handleWithdrawModal,
      activeTokens,
      activeOrderbookAddresses,
      showMyItemsOnly: writable(false)
    },
    {},
    {}
  )}`;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
