import { c as create_ssr_component, a as compute_rest_props, b as spread, d as escape_object, e as escape_attribute_value, v as validate_component, h as escape, p as onDestroy, f as add_attribute, j as each, k as subscribe, o as noop, n as getAllContexts } from "../../../../chunks/ssr.js";
import { p as page } from "../../../../chunks/stores.js";
import { l as QKEY_VAULT_CHANGES, D as DEFAULT_PAGE_SIZE, T as TanstackAppTable, u as useAccount, m as QKEY_VAULT, i as handleDepositModal, j as handleWithdrawModal } from "../../../../chunks/modal.js";
import { createInfiniteQuery, useQueryClient, createQuery } from "@tanstack/svelte-query";
import { i as invalidateTanstackQueries, T as TanstackPageContentDetail, C as CardProperty, g as getExplorerLink, A as ArrowDownToBracketOutline } from "../../../../chunks/getExplorerLink.js";
import { P as PageHeader } from "../../../../chunks/PageHeader.js";
import { H as Hash, e as HashType, u as useRaindexClient, W as WalletOutline } from "../../../../chunks/queryClient.js";
import { T as TableBodyCell, a as TableHeadCell, R as Refresh } from "../../../../chunks/order.js";
import { i as is_void, B as Button } from "../../../../chunks/darkMode.js";
import { twMerge } from "tailwind-merge";
import "../../../../chunks/sentry.js";
import { f as formatTimestampSecondsAsLocal } from "../../../../chunks/time.js";
import { toHex } from "viem";
import { O as OrderOrVaultHash } from "../../../../chunks/OrderOrVaultHash.js";
import { A as ArrowUpFromBracketOutline } from "../../../../chunks/ArrowUpFromBracketOutline.js";
import { u as useToasts } from "../../../../chunks/useToasts.js";
const Heading = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["tag", "color", "customSize"]);
  let { tag = "h1" } = $$props;
  let { color = "text-gray-900 dark:text-white" } = $$props;
  let { customSize = "" } = $$props;
  const textSizes = {
    h1: "text-5xl font-extrabold",
    h2: "text-4xl font-bold",
    h3: "text-3xl font-bold",
    h4: "text-2xl font-bold",
    h5: "text-xl font-bold",
    h6: "text-lg font-bold"
  };
  if ($$props.tag === void 0 && $$bindings.tag && tag !== void 0) $$bindings.tag(tag);
  if ($$props.color === void 0 && $$bindings.color && color !== void 0) $$bindings.color(color);
  if ($$props.customSize === void 0 && $$bindings.customSize && customSize !== void 0) $$bindings.customSize(customSize);
  return `${((tag$1) => {
    return tag$1 ? `<${tag}${spread(
      [
        escape_object($$restProps),
        {
          class: escape_attribute_value(twMerge(customSize ? customSize : textSizes[tag], color, "w-full", $$props.class))
        }
      ],
      {}
    )}>${is_void(tag$1) ? "" : `${slots.default ? slots.default({}) : ``}`}${is_void(tag$1) ? "" : `</${tag$1}>`}` : "";
  })(tag)} `;
});
const VaultBalanceChangesTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let balanceChangesQuery;
  let { vault } = $$props;
  const AppTable = TanstackAppTable;
  if ($$props.vault === void 0 && $$bindings.vault && vault !== void 0) $$bindings.vault(vault);
  balanceChangesQuery = createInfiniteQuery({
    queryKey: [vault.id, QKEY_VAULT_CHANGES + vault.id],
    queryFn: async ({ pageParam }) => {
      const result = await vault.getBalanceChanges(pageParam + 1);
      if (result.error) throw new Error(result.error.msg);
      return result.value;
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : void 0;
    }
  });
  return `${validate_component(AppTable, "AppTable").$$render(
    $$result,
    {
      query: balanceChangesQuery,
      queryKey: vault.id,
      emptyMessage: "No deposits or withdrawals found",
      rowHoverable: false
    },
    {},
    {
      bodyRow: ({ item }) => {
        return `${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "px-4 py-2",
            "data-testid": "vaultBalanceChangesTableDate"
          },
          {},
          {
            default: () => {
              return `${escape(formatTimestampSecondsAsLocal(BigInt(item.timestamp)))}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all py-2 min-w-48",
            "data-testid": "vaultBalanceChangesTableFrom"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Wallet,
                  value: item.transaction.from
                },
                {},
                {}
              )}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-all py-2 min-w-48",
            "data-testid": "vaultBalanceChangesTableTx"
          },
          {},
          {
            default: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Transaction,
                  value: item.transaction.id
                },
                {},
                {}
              )}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-word p-0 text-left",
            "data-testid": "vaultBalanceChangesTableBalanceChange"
          },
          {},
          {
            default: () => {
              return `${escape(`${item.formattedAmount} ${item.token.symbol}`)}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-word p-0 text-left",
            "data-testid": "vaultBalanceChangesTableBalance"
          },
          {},
          {
            default: () => {
              return `${escape(`${item.formattedNewBalance} ${item.token.symbol}`)}`;
            }
          }
        )} ${validate_component(TableBodyCell, "TableBodyCell").$$render(
          $$result,
          {
            tdClass: "break-word p-0 text-left",
            "data-testid": "vaultBalanceChangesTableType"
          },
          {},
          {
            default: () => {
              return `${escape(item.type)}`;
            }
          }
        )} `;
      },
      head: () => {
        return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-4" }, {}, {
          default: () => {
            return `Date`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
          default: () => {
            return `Sender`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
          default: () => {
            return `Transaction Hash`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
          default: () => {
            return `Balance Change`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p-0" }, {}, {
          default: () => {
            return `Balance`;
          }
        })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, { padding: "p--" }, {}, {
          default: () => {
            return `Type`;
          }
        })} `;
      },
      title: () => {
        return `${validate_component(Heading, "Heading").$$render(
          $$result,
          {
            tag: "h5",
            class: "mb-4 mt-6 font-normal"
          },
          {},
          {
            default: () => {
              return `Vault balance changes`;
            }
          }
        )}`;
      }
    }
  )}`;
});
const VaultDetail = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let vaultDetailQuery;
  let $vaultDetailQuery, $$unsubscribe_vaultDetailQuery = noop, $$subscribe_vaultDetailQuery = () => ($$unsubscribe_vaultDetailQuery(), $$unsubscribe_vaultDetailQuery = subscribe(vaultDetailQuery, ($$value) => $vaultDetailQuery = $$value), vaultDetailQuery);
  let { id } = $$props;
  let { orderbookAddress } = $$props;
  let { chainId } = $$props;
  let { onDeposit } = $$props;
  let { onWithdraw } = $$props;
  const queryClient = useQueryClient();
  const { matchesAccount } = useAccount();
  useToasts();
  const raindexClient = useRaindexClient();
  const interval = setInterval(
    async () => {
      invalidateTanstackQueries(queryClient, [id, QKEY_VAULT + id]);
    },
    5e3
  );
  onDestroy(() => {
    clearInterval(interval);
  });
  if ($$props.id === void 0 && $$bindings.id && id !== void 0) $$bindings.id(id);
  if ($$props.orderbookAddress === void 0 && $$bindings.orderbookAddress && orderbookAddress !== void 0) $$bindings.orderbookAddress(orderbookAddress);
  if ($$props.chainId === void 0 && $$bindings.chainId && chainId !== void 0) $$bindings.chainId(chainId);
  if ($$props.onDeposit === void 0 && $$bindings.onDeposit && onDeposit !== void 0) $$bindings.onDeposit(onDeposit);
  if ($$props.onWithdraw === void 0 && $$bindings.onWithdraw && onWithdraw !== void 0) $$bindings.onWithdraw(onWithdraw);
  $$subscribe_vaultDetailQuery(vaultDetailQuery = createQuery({
    queryKey: [id, QKEY_VAULT + id],
    queryFn: async () => {
      const result = await raindexClient.getVault(chainId, orderbookAddress, id);
      if (result.error) throw new Error(result.error.readableMsg);
      return result.value;
    }
  }));
  $$unsubscribe_vaultDetailQuery();
  return `${validate_component(TanstackPageContentDetail, "TanstackPageContentDetail").$$render(
    $$result,
    {
      query: vaultDetailQuery,
      emptyMessage: "Vault not found"
    },
    {},
    {
      below: ({ data }) => {
        return `${validate_component(VaultBalanceChangesTable, "VaultBalanceChangesTable").$$render($$result, { vault: data }, {}, {})}`;
      },
      chart: () => {
        return `  `;
      },
      card: ({ data }) => {
        return `${validate_component(CardProperty, "CardProperty").$$render($$result, { "data-testid": "vaultDetailVaultId" }, {}, {
          value: () => {
            return `${escape(toHex(data.vaultId))}`;
          },
          key: () => {
            return `Vault ID`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render(
          $$result,
          {
            "data-testid": "vaultDetailOrderbookAddress"
          },
          {},
          {
            value: () => {
              return `${validate_component(Hash, "Hash").$$render(
                $$result,
                {
                  type: HashType.Identifier,
                  value: data.orderbook
                },
                {},
                {}
              )} `;
            },
            key: () => {
              return `Orderbook`;
            }
          }
        )} ${validate_component(CardProperty, "CardProperty").$$render($$result, { "data-testid": "vaultDetailOwnerAddress" }, {}, {
          value: () => {
            let explorerLink = getExplorerLink(data.owner, chainId, "address");
            return `${explorerLink ? `<a${add_attribute("href", explorerLink, 0)} target="_blank" rel="noopener noreferrer" class="flex items-center justify-start space-x-2 text-left text-blue-500 hover:underline">${validate_component(WalletOutline, "WalletOutline").$$render($$result, { size: "sm" }, {}, {})} <span>${escape(data.owner)}</span></a>` : `${validate_component(Hash, "Hash").$$render($$result, { type: HashType.Wallet, value: data.owner }, {}, {})}`} `;
          },
          key: () => {
            return `Owner address`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, { "data-testid": "vaultDetailTokenAddress" }, {}, {
          value: () => {
            return `${validate_component(Hash, "Hash").$$render($$result, { value: data.token.id }, {}, {})} `;
          },
          key: () => {
            return `Token address`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, { "data-testid": "vaultDetailBalance" }, {}, {
          value: () => {
            return `${escape(`${data.formattedBalance} ${data.token.symbol}`)}`;
          },
          key: () => {
            return `Balance`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
          value: () => {
            return `<p data-testid="vaultDetailOrdersAsInput" class="flex flex-wrap justify-start">${data.ordersAsInput && data.ordersAsInput.length > 0 ? `${each(data.ordersAsInput, (order) => {
              return `${validate_component(OrderOrVaultHash, "OrderOrVaultHash").$$render(
                $$result,
                {
                  type: "orders",
                  orderOrVault: order,
                  chainId,
                  orderbookAddress
                },
                {},
                {}
              )}`;
            })}` : `None`}</p> `;
          },
          key: () => {
            return `Orders as input`;
          }
        })} ${validate_component(CardProperty, "CardProperty").$$render($$result, {}, {}, {
          value: () => {
            return `<p data-testid="vaultDetailOrdersAsOutput" class="flex flex-wrap justify-start">${data.ordersAsOutput && data.ordersAsOutput.length > 0 ? `${each(data.ordersAsOutput, (order) => {
              return `${validate_component(OrderOrVaultHash, "OrderOrVaultHash").$$render(
                $$result,
                {
                  type: "orders",
                  orderOrVault: order,
                  chainId,
                  orderbookAddress
                },
                {},
                {}
              )}`;
            })}` : `None`}</p> `;
          },
          key: () => {
            return `Orders as output`;
          }
        })} `;
      },
      top: ({ data }) => {
        return `<div data-testid="vaultDetailTokenName" class="flex gap-x-4 text-3xl font-medium dark:text-white">${escape(data.token.name)}</div> <div class="flex items-center gap-2">${matchesAccount(data.owner) ? `${validate_component(Button, "Button").$$render(
          $$result,
          {
            color: "light",
            size: "xs",
            "data-testid": "deposit-button",
            "aria-label": "Deposit to vault"
          },
          {},
          {
            default: () => {
              return `${validate_component(ArrowDownToBracketOutline, "ArrowDownToBracketOutline").$$render($$result, { size: "xs" }, {}, {})}`;
            }
          }
        )} ${validate_component(Button, "Button").$$render(
          $$result,
          {
            color: "light",
            size: "xs",
            "data-testid": "withdraw-button",
            "aria-label": "Withdraw from vault"
          },
          {},
          {
            default: () => {
              return `${validate_component(ArrowUpFromBracketOutline, "ArrowUpFromBracketOutline").$$render($$result, { size: "xs" }, {}, {})}`;
            }
          }
        )}` : ``} ${validate_component(Refresh, "Refresh").$$render(
          $$result,
          {
            testId: "top-refresh",
            spin: $vaultDetailQuery.isLoading || $vaultDetailQuery.isFetching
          },
          {},
          {}
        )}</div>`;
      }
    }
  )}`;
});
const Page = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $page, $$unsubscribe_page;
  $$unsubscribe_page = subscribe(page, (value) => $page = value);
  const context = getAllContexts();
  const { chainId, orderbook, id } = $page.params;
  const parsedId = id;
  const parsedChainId = Number(chainId);
  const orderbookAddress = orderbook;
  const queryClient = useQueryClient();
  function onDeposit(_raindexClient, vault) {
    handleDepositModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [$page.params.id]);
      },
      context
    );
  }
  function onWithdraw(_raindexClient, vault) {
    handleWithdrawModal(
      vault,
      () => {
        invalidateTanstackQueries(queryClient, [$page.params.id]);
      },
      context
    );
  }
  $$unsubscribe_page();
  return `${validate_component(PageHeader, "PageHeader").$$render(
    $$result,
    {
      title: "Vault",
      pathname: $page.url.pathname
    },
    {},
    {}
  )} ${validate_component(VaultDetail, "VaultDetail").$$render(
    $$result,
    {
      chainId: parsedChainId,
      orderbookAddress,
      id: parsedId,
      onDeposit,
      onWithdraw
    },
    {},
    {}
  )}`;
});
export {
  Page as default
};
//# sourceMappingURL=_page.svelte.js.map
