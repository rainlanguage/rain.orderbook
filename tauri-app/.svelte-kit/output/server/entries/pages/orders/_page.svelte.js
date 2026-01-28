import { c as create_ssr_component, h as escape, n as getAllContexts, k as subscribe, v as validate_component, j as each, o as noop } from "../../../chunks/ssr.js";
import { u as useAccount, Q as QKEY_TOKENS, a as QKEY_ORDERS, D as DEFAULT_PAGE_SIZE, b as DEFAULT_REFRESH_INTERVAL, T as TanstackAppTable, h as handleOrderRemoveModal } from "../../../chunks/modal.js";
import { c as selectedChainIds, d as activeAccountsItems, e as showInactiveOrders, o as orderHash, h as hideZeroBalanceVaults, f as hideInactiveOrdersVaults, g as activeTokens, i as activeOrderbookAddresses } from "../../../chunks/sentry.js";
import { p as page } from "../../../chunks/stores.js";
import { w as writable } from "../../../chunks/index.js";
import { P as PageHeader } from "../../../chunks/PageHeader.js";
import { T as TableBodyCell, g as getNetworkName, a as TableHeadCell } from "../../../chunks/order.js";
import { L as ListViewOrderbookFilters, D as DotsVerticalOutline, a as DropdownItem } from "../../../chunks/ListViewOrderbookFilters.js";
import { createQuery, createInfiniteQuery } from "@tanstack/svelte-query";
import { f as formatTimestampSecondsAsLocal } from "../../../chunks/time.js";
import { u as useRaindexClient, H as Hash, e as HashType } from "../../../chunks/queryClient.js";
import { B as Badge } from "../../../chunks/Badge.js";
import { B as Button } from "../../../chunks/darkMode.js";
import { D as Dropdown } from "../../../chunks/ChevronDownSolid.js";
const VaultCard = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { vault } = $$props;
  if ($$props.vault === void 0 && $$bindings.vault && vault !== void 0) $$bindings.vault(vault);
  return `<button type="button" class="flex flex-col rounded-xl border border-gray-200 bg-gray-50 px-4 py-3 text-left shadow-sm transition-colors hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:hover:bg-gray-600" data-testid="vault-card"><span class="font-semibold text-gray-800 dark:text-gray-200">${escape(vault.token.symbol)}</span> <span class="truncate text-xs text-gray-500 dark:text-gray-400">${escape(vault.formattedBalance)}</span></button>`;
});
const OrdersListTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let owners;
  let tokensQuery;
  let selectedTokens;
  let orderbooksMap;
  let availableOrderbookAddresses;
  let selectedOrderbookAddresses;
  let query;
  let $orderHash, $$unsubscribe_orderHash;
  let $showInactiveOrders, $$unsubscribe_showInactiveOrders;
  let $selectedChainIds, $$unsubscribe_selectedChainIds;
  let $activeOrderbookAddresses, $$unsubscribe_activeOrderbookAddresses;
  let $tokensQuery, $$unsubscribe_tokensQuery = noop, $$subscribe_tokensQuery = () => ($$unsubscribe_tokensQuery(), $$unsubscribe_tokensQuery = subscribe(tokensQuery, ($$value) => $tokensQuery = $$value), tokensQuery);
  let $activeTokens, $$unsubscribe_activeTokens;
  let $account, $$unsubscribe_account;
  let $showMyItemsOnly, $$unsubscribe_showMyItemsOnly;
  let $activeAccountsItems, $$unsubscribe_activeAccountsItems;
  let $$unsubscribe_query = noop, $$subscribe_query = () => ($$unsubscribe_query(), $$unsubscribe_query = subscribe(query, ($$value) => $$value), query);
  getAllContexts();
  let { handleOrderRemoveModal: handleOrderRemoveModal2 = void 0 } = $$props;
  let { selectedChainIds: selectedChainIds2 } = $$props;
  $$unsubscribe_selectedChainIds = subscribe(selectedChainIds2, (value) => $selectedChainIds = value);
  let { activeAccountsItems: activeAccountsItems2 } = $$props;
  $$unsubscribe_activeAccountsItems = subscribe(activeAccountsItems2, (value) => $activeAccountsItems = value);
  let { showInactiveOrders: showInactiveOrders2 } = $$props;
  $$unsubscribe_showInactiveOrders = subscribe(showInactiveOrders2, (value) => $showInactiveOrders = value);
  let { orderHash: orderHash2 } = $$props;
  $$unsubscribe_orderHash = subscribe(orderHash2, (value) => $orderHash = value);
  let { hideZeroBalanceVaults: hideZeroBalanceVaults2 } = $$props;
  let { hideInactiveOrdersVaults: hideInactiveOrdersVaults2 } = $$props;
  let { showMyItemsOnly } = $$props;
  $$unsubscribe_showMyItemsOnly = subscribe(showMyItemsOnly, (value) => $showMyItemsOnly = value);
  let { activeTokens: activeTokens2 } = $$props;
  $$unsubscribe_activeTokens = subscribe(activeTokens2, (value) => $activeTokens = value);
  let { activeOrderbookAddresses: activeOrderbookAddresses2 } = $$props;
  $$unsubscribe_activeOrderbookAddresses = subscribe(activeOrderbookAddresses2, (value) => $activeOrderbookAddresses = value);
  const { matchesAccount, account } = useAccount();
  $$unsubscribe_account = subscribe(account, (value) => $account = value);
  const raindexClient = useRaindexClient();
  const AppTable = TanstackAppTable;
  if ($$props.handleOrderRemoveModal === void 0 && $$bindings.handleOrderRemoveModal && handleOrderRemoveModal2 !== void 0) $$bindings.handleOrderRemoveModal(handleOrderRemoveModal2);
  if ($$props.selectedChainIds === void 0 && $$bindings.selectedChainIds && selectedChainIds2 !== void 0) $$bindings.selectedChainIds(selectedChainIds2);
  if ($$props.activeAccountsItems === void 0 && $$bindings.activeAccountsItems && activeAccountsItems2 !== void 0) $$bindings.activeAccountsItems(activeAccountsItems2);
  if ($$props.showInactiveOrders === void 0 && $$bindings.showInactiveOrders && showInactiveOrders2 !== void 0) $$bindings.showInactiveOrders(showInactiveOrders2);
  if ($$props.orderHash === void 0 && $$bindings.orderHash && orderHash2 !== void 0) $$bindings.orderHash(orderHash2);
  if ($$props.hideZeroBalanceVaults === void 0 && $$bindings.hideZeroBalanceVaults && hideZeroBalanceVaults2 !== void 0) $$bindings.hideZeroBalanceVaults(hideZeroBalanceVaults2);
  if ($$props.hideInactiveOrdersVaults === void 0 && $$bindings.hideInactiveOrdersVaults && hideInactiveOrdersVaults2 !== void 0) $$bindings.hideInactiveOrdersVaults(hideInactiveOrdersVaults2);
  if ($$props.showMyItemsOnly === void 0 && $$bindings.showMyItemsOnly && showMyItemsOnly !== void 0) $$bindings.showMyItemsOnly(showMyItemsOnly);
  if ($$props.activeTokens === void 0 && $$bindings.activeTokens && activeTokens2 !== void 0) $$bindings.activeTokens(activeTokens2);
  if ($$props.activeOrderbookAddresses === void 0 && $$bindings.activeOrderbookAddresses && activeOrderbookAddresses2 !== void 0) $$bindings.activeOrderbookAddresses(activeOrderbookAddresses2);
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
      QKEY_ORDERS,
      $selectedChainIds,
      owners,
      $showInactiveOrders,
      $orderHash,
      selectedTokens,
      selectedOrderbookAddresses
    ],
    queryFn: async ({ pageParam }) => {
      const result = await raindexClient.getOrders(
        $selectedChainIds,
        {
          owners,
          active: $showInactiveOrders ? void 0 : true,
          orderHash: $orderHash || void 0,
          tokens: selectedTokens.length > 0 ? {
            inputs: selectedTokens,
            outputs: selectedTokens
          } : void 0,
          orderbookAddresses: selectedOrderbookAddresses.length > 0 ? selectedOrderbookAddresses : void 0
        },
        pageParam + 1
      );
      if (result.error) throw new Error(result.error.readableMsg);
      return result.value;
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : void 0;
    },
    refetchInterval: DEFAULT_REFRESH_INTERVAL,
    enabled: true
  }));
  $$unsubscribe_orderHash();
  $$unsubscribe_showInactiveOrders();
  $$unsubscribe_selectedChainIds();
  $$unsubscribe_activeOrderbookAddresses();
  $$unsubscribe_tokensQuery();
  $$unsubscribe_activeTokens();
  $$unsubscribe_account();
  $$unsubscribe_showMyItemsOnly();
  $$unsubscribe_activeAccountsItems();
  $$unsubscribe_query();
  return `${validate_component(ListViewOrderbookFilters, "ListViewOrderbookFilters").$$render(
    $$result,
    {
      selectedChainIds: selectedChainIds2,
      activeAccountsItems: activeAccountsItems2,
      showMyItemsOnly,
      showInactiveOrders: showInactiveOrders2,
      orderHash: orderHash2,
      hideZeroBalanceVaults: hideZeroBalanceVaults2,
      hideInactiveOrdersVaults: hideInactiveOrdersVaults2,
      tokensQuery,
      activeTokens: activeTokens2,
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
      queryKey: QKEY_ORDERS,
      emptyMessage: "No Orders Found"
    },
    {},
    {
      bodyRow: ({ item }) => {
        return `${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowNetwork",
            tdClass: "px-4 py-2"
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
            "data-testid": "orderListRowActive",
            tdClass: "px-4 py-2"
          },
          {},
          {
            default: () => {
              return `${item.active ? `${validate_component(Badge, "Badge").$$render($$result, { color: "green" }, {}, {
                default: () => {
                  return `Active`;
                }
              })}` : `${validate_component(Badge, "Badge").$$render($$result, { color: "yellow" }, {}, {
                default: () => {
                  return `Inactive`;
                }
              })}`}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowID",
            tdClass: "break-all px-4 py-4"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Identifier,
                  value: item.orderHash
                },
                {},
                {}
              )}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowOwner",
            tdClass: "break-all px-4 py-2"
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
            "data-testid": "orderListRowOrderbook",
            tdClass: "break-all px-4 py-2"
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
            "data-testid": "orderListRowLastAdded",
            tdClass: "break-word px-4 py-2"
          },
          {},
          {
            default: () => {
              return `${escape(formatTimestampSecondsAsLocal(item.timestampAdded))}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowInputs",
            tdClass: "p-2 whitespace-normal"
          },
          {},
          {
            default: () => {
              return `<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">${each(item.inputsList.items, (vault) => {
                return `${validate_component(VaultCard, "VaultCard").$$render($$result, { vault }, {}, {})}`;
              })} ${each(item.inputsOutputsList.items, (vault) => {
                return `${!item.inputsList.items.find((v) => v.id === vault.id) ? `${validate_component(VaultCard, "VaultCard").$$render($$result, { vault }, {}, {})}` : ``}`;
              })}</div>`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowOutputs",
            tdClass: "p-2 whitespace-normal"
          },
          {},
          {
            default: () => {
              return `<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">${each(item.outputsList.items, (vault) => {
                return `${validate_component(VaultCard, "VaultCard").$$render($$result, { vault }, {}, {})}`;
              })} ${each(item.inputsOutputsList.items, (vault) => {
                return `${!item.outputsList.items.find((v) => v.id === vault.id) ? `${validate_component(VaultCard, "VaultCard").$$render($$result, { vault }, {}, {})}` : ``}`;
              })}</div>`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            "data-testid": "orderListRowTrades",
            tdClass: "break-word p-2"
          },
          {},
          {
            default: () => {
              return `${escape(item.tradesCount > 99 ? ">99" : item.tradesCount)}`;
            }
          }
        )} ${matchesAccount(item.owner) && handleOrderRemoveModal2 ? `<div data-testid="wallet-actions">${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { tdClass: "px-0 text-right" }, {}, {
          default: () => {
            return `${item.active ? `${validate_component(Button, "Button").$$render(
              $$result,
              {
                color: "alternative",
                outline: false,
                "data-testid": `order-menu-${item.id}`,
                id: `order-menu-${item.id}`,
                class: "mr-2 border-none px-2"
              },
              {},
              {
                default: () => {
                  return `${validate_component(DotsVerticalOutline, "DotsVerticalOutline").$$render($$result, { class: "dark:text-white" }, {}, {})}`;
                }
              }
            )}` : ``}`;
          }
        })} ${item.active ? `${validate_component(Dropdown, "Dropdown").$$render(
          $$result,
          {
            placement: "bottom-end",
            triggeredBy: `#order-menu-${item.id}`
          },
          {},
          {
            default: () => {
              return `${validate_component(DropdownItem, "DropdownItem").$$render($$result, {}, {}, {
                default: () => {
                  return `Remove`;
                }
              })}`;
            }
          }
        )}` : ``}</div>` : ``} `;
      },
      head: () => {
        return `${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingNetwork",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Network`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingActive",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Active`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingID",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Order`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingOwner",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Owner`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingOrderbook",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Orderbook`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingLastAdded",
            padding: "p-4"
          },
          {},
          {
            default: () => {
              return `Last Added`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingInputs",
            padding: "px-2 py-4"
          },
          {},
          {
            default: () => {
              return `Input Token(s)`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingOutputs",
            padding: "px-2 py-4"
          },
          {},
          {
            default: () => {
              return `Output Token(s)`;
            }
          }
        )} ${validate_component(TableHeadCell, "TableHeadCell").$$render(
          $$result,
          {
            "data-testid": "orderListHeadingTrades",
            padding: "px-2 py-4"
          },
          {},
          {
            default: () => {
              return `Trades`;
            }
          }
        )} `;
      },
      title: () => {
        return `<div class="mt-2 flex w-full justify-between"><div class="text-3xl font-medium dark:text-white" data-testid="title" data-svelte-h="svelte-15h8ijg">Orders</div> ${slots.filters ? slots.filters({}) : ``}</div>`;
      }
    }
  )}`;
});
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  $$unsubscribe_page();
  return `${validate_component(PageHeader, "PageHeader").$$render(
    $$result,
    {
      title: "Orders",
      pathname: $page.url.pathname
    },
    {},
    {}
  )} ${validate_component(OrdersListTable, "OrdersListTable").$$render(
    $$result,
    {
      handleOrderRemoveModal,
      selectedChainIds,
      activeAccountsItems,
      showInactiveOrders,
      orderHash,
      hideZeroBalanceVaults,
      hideInactiveOrdersVaults,
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
