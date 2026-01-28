import { c as create_ssr_component, a as compute_rest_props, s as setContext, b as spread, d as escape_object, e as escape_attribute_value, g as getContext, l as createEventDispatcher, k as subscribe, p as onDestroy, v as validate_component, h as escape, f as add_attribute, j as each, q as get_store_value, o as noop } from "./ssr.js";
import { j as clampSize, R as Refresh, b as Table, c as TableHead, d as TableBody, e as TableBodyRow, L as Label, M as ModalExecute, i as ethersExecute, k as orderRemove, a as TableHeadCell, T as TableBodyCell } from "./order.js";
import { B as Button } from "./darkMode.js";
import { useQueryClient, createQuery } from "@tanstack/svelte-query";
import { isAddress, isAddressEqual, toHex, hexToBytes, isHex, formatEther, hexToBigInt } from "viem";
import { g as getAccountContext } from "./context.js";
import { twMerge } from "tailwind-merge";
import { h as walletConnectNetwork, k as ledgerWalletDerivationIndex, u as useRaindexClient, M as Modal, i as formatEthersTransactionError, m as Helper, q as queryClient } from "./queryClient.js";
import { m as Float, r as reportErrorToSentry, k as toasts, S as Spinner, A as Alert } from "./sentry.js";
import { invoke } from "@tauri-apps/api";
import "@tauri-apps/api/mocks";
const ButtonGroup = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $$restProps = compute_rest_props($$props, ["size", "divClass"]);
  let { size = "md" } = $$props;
  let { divClass = "inline-flex rounded-lg shadow-sm" } = $$props;
  setContext("group", { size });
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  if ($$props.divClass === void 0 && $$bindings.divClass && divClass !== void 0) $$bindings.divClass(divClass);
  return `<div${spread(
    [
      escape_object($$restProps),
      {
        class: escape_attribute_value(twMerge(divClass, $$props.class))
      },
      { role: "group" }
    ],
    {}
  )}>${slots.default ? slots.default({}) : ``}</div> `;
});
const InputAddon = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let _size;
  let divClass;
  let $$restProps = compute_rest_props($$props, ["size"]);
  let { size = void 0 } = $$props;
  let background = getContext("background");
  let group = getContext("group");
  const borderClasses = {
    base: "border-gray-300 dark:border-gray-600",
    tinted: "border-gray-300 dark:border-gray-500"
  };
  const darkBgClasses = {
    base: "dark:bg-gray-600 dark:text-gray-400",
    tinted: "dark:bg-gray-500 dark:text-gray-300"
  };
  const divider = {
    base: "dark:border-e-gray-700 dark:last:border-e-gray-600",
    tinted: "dark:border-e-gray-600 dark:last:border-e-gray-500"
  };
  const textSizes = {
    sm: "sm:text-xs",
    md: "text-sm",
    lg: "sm:text-base"
  };
  const prefixPadding = { sm: "px-2", md: "px-3", lg: "px-4" };
  if ($$props.size === void 0 && $$bindings.size && size !== void 0) $$bindings.size(size);
  _size = size || clampSize(group?.size) || "md";
  divClass = twMerge(
    textSizes[_size],
    prefixPadding[_size],
    background ? borderClasses["tinted"] : borderClasses["base"],
    "text-gray-500 bg-gray-200",
    background ? darkBgClasses.tinted : darkBgClasses.base,
    background ? divider.tinted : divider.base,
    "inline-flex items-center border-t border-b first:border-s border-e",
    "first:rounded-s-lg last:rounded-e-lg",
    $$props.class
  );
  return `<div${spread([escape_object($$restProps), { class: escape_attribute_value(divClass) }], {})}>${slots.default ? slots.default({}) : ``}</div> `;
});
const TanstackAppTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let enableVirtualization;
  let flattenedRows;
  let totalRows;
  let hasData;
  let $query, $$unsubscribe_query;
  useQueryClient();
  createEventDispatcher();
  let { queryKey } = $$props;
  let { query } = $$props;
  $$unsubscribe_query = subscribe(query, (value) => $query = value);
  let { emptyMessage = "None found" } = $$props;
  let { rowHoverable = true } = $$props;
  let { dataSelector = (pageData) => Array.isArray(pageData) ? pageData : [] } = $$props;
  let { virtualization = {} } = $$props;
  let transformedPages = [];
  let lastPagesRef;
  let lastSelector = dataSelector;
  let tableContainerElement = null;
  let tableOffsetTop = 0;
  let virtualizer = null;
  let virtualizationActive = false;
  let virtualItems = [];
  let totalSize = 0;
  let topPadding = 0;
  let bottomPadding = 0;
  let scrollMargin = 0;
  onDestroy(() => {
  });
  if ($$props.queryKey === void 0 && $$bindings.queryKey && queryKey !== void 0) $$bindings.queryKey(queryKey);
  if ($$props.query === void 0 && $$bindings.query && query !== void 0) $$bindings.query(query);
  if ($$props.emptyMessage === void 0 && $$bindings.emptyMessage && emptyMessage !== void 0) $$bindings.emptyMessage(emptyMessage);
  if ($$props.rowHoverable === void 0 && $$bindings.rowHoverable && rowHoverable !== void 0) $$bindings.rowHoverable(rowHoverable);
  if ($$props.dataSelector === void 0 && $$bindings.dataSelector && dataSelector !== void 0) $$bindings.dataSelector(dataSelector);
  if ($$props.virtualization === void 0 && $$bindings.virtualization && virtualization !== void 0) $$bindings.virtualization(virtualization);
  enableVirtualization = virtualization.enabled ?? true;
  virtualization.estimatedRowHeight ?? 56;
  virtualization.overscan ?? 8;
  {
    {
      const currentData = $query.data;
      const currentPages = currentData?.pages;
      const selectorChanged = lastSelector !== dataSelector;
      if (!currentPages) {
        transformedPages = [];
        lastPagesRef = void 0;
        lastSelector = dataSelector;
      } else if (currentPages !== lastPagesRef || selectorChanged) {
        transformedPages = currentPages.map((page) => dataSelector(page));
        lastPagesRef = currentPages;
        lastSelector = dataSelector;
      }
    }
  }
  flattenedRows = transformedPages.flat();
  totalRows = flattenedRows.length;
  hasData = totalRows > 0;
  virtualizationActive = enableVirtualization && Boolean(virtualizer);
  {
    {
      const hasRows = totalRows > 0;
      scrollMargin = virtualizationActive ? tableOffsetTop : 0;
      if (virtualizationActive && virtualizer && hasRows) {
        virtualItems = virtualizer.getVirtualItems();
        totalSize = virtualizer.getTotalSize();
        const firstItem = virtualItems[0];
        const lastItem = virtualItems[virtualItems.length - 1];
        topPadding = firstItem ? Math.max(0, firstItem.start - scrollMargin) : 0;
        bottomPadding = lastItem ? Math.max(0, totalSize - (lastItem.end - scrollMargin)) : Math.max(0, totalSize);
      } else {
        virtualItems = [];
        totalSize = 0;
        topPadding = 0;
        bottomPadding = 0;
      }
    }
  }
  $$unsubscribe_query();
  return `<div data-testid="title" class="flex h-16 w-full items-center justify-end">${slots.info ? slots.info({}) : ``} ${slots.timeFilter ? slots.timeFilter({}) : ``} ${slots.title ? slots.title({}) : ``} ${validate_component(Refresh, "Refresh").$$render(
    $$result,
    {
      class: "ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400",
      "data-testid": "refreshButton",
      spin: $query.isLoading || $query.isFetching
    },
    {},
    {}
  )}</div> ${totalRows === 0 ? `<div data-testid="emptyMessage" class="text-center text-gray-900 dark:text-white">${escape(emptyMessage)}</div>` : `${hasData ? `<div class="cursor-pointer overflow-x-auto rounded-lg border dark:border-none" data-testid="tanstackTableContainer"${add_attribute("this", tableContainerElement, 0)}>${validate_component(Table, "Table").$$render(
    $$result,
    {
      divClass: "min-w-full",
      hoverable: rowHoverable
    },
    {},
    {
      default: () => {
        return `${validate_component(TableHead, "TableHead").$$render($$result, { "data-testid": "head" }, {}, {
          default: () => {
            return `${slots.head ? slots.head({}) : ``}`;
          }
        })} ${validate_component(TableBody, "TableBody").$$render($$result, {}, {}, {
          default: () => {
            return `${virtualizationActive && topPadding > 0 ? `<tr aria-hidden="true"><td colspan="1000" class="border-0 p-0"${add_attribute("style", `height:${topPadding}px;`, 0)}></td></tr>` : ``} ${virtualizationActive ? `${each(virtualItems, (virtualItem) => {
              return `${validate_component(TableBodyRow, "TableBodyRow").$$render(
                $$result,
                {
                  class: "whitespace-nowrap",
                  "data-testid": "bodyRow",
                  "data-virtual-row": "true"
                },
                {},
                {
                  default: () => {
                    return `${slots.bodyRow ? slots.bodyRow({ item: flattenedRows[virtualItem.index] }) : ``} `;
                  }
                }
              )}`;
            })}` : `${each(transformedPages, (page) => {
              return `${each(page, (item) => {
                return `${validate_component(TableBodyRow, "TableBodyRow").$$render(
                  $$result,
                  {
                    class: "whitespace-nowrap",
                    "data-testid": "bodyRow"
                  },
                  {},
                  {
                    default: () => {
                      return `${slots.bodyRow ? slots.bodyRow({ item }) : ``} `;
                    }
                  }
                )}`;
              })}`;
            })}`} ${virtualizationActive && bottomPadding > 0 ? `<tr aria-hidden="true"><td colspan="1000" class="border-0 p-0"${add_attribute("style", `height:${bottomPadding}px;`, 0)}></td></tr>` : ``}`;
          }
        })}`;
      }
    }
  )}</div> <div class="mt-2 flex justify-center">${validate_component(Button, "Button").$$render(
    $$result,
    {
      "data-testid": "loadMoreButton",
      size: "xs",
      color: "dark",
      disabled: !$query.hasNextPage || $query.isFetchingNextPage
    },
    {},
    {
      default: () => {
        return `${$query.isFetchingNextPage ? `Loading more...` : `${$query.hasNextPage ? `Load More` : `Nothing more to load`}`}`;
      }
    }
  )}</div>` : ``}`}`;
});
function useAccount() {
  const account = getAccountContext();
  const matchesAccount = (otherAddress) => {
    if (!otherAddress)
      return false;
    const currentAccount = get_store_value(account);
    if (!currentAccount) {
      return false;
    }
    if (isAddress(currentAccount) && isAddress(otherAddress) && isAddressEqual(currentAccount, otherAddress)) {
      return true;
    }
    return false;
  };
  return {
    account,
    matchesAccount
  };
}
const DEFAULT_PAGE_SIZE = 50;
const DEFAULT_REFRESH_INTERVAL = 1e4;
const QKEY_VAULTS = "vaults";
const QKEY_VAULT = "vault";
const QKEY_VAULT_CHANGES = "vaultBalanceChanges";
const QKEY_ORDERS = "orders";
const QKEY_ORDER = "order";
const QKEY_ORDER_TRADES_LIST = "orderTradesList";
const QKEY_ORDER_QUOTE = "orderQuote";
const QKEY_TOKENS = "tokens";
const InputTokenAmount = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { symbol = void 0 } = $$props;
  let { maxValue = void 0 } = $$props;
  let { value = Float.parse("0").value } = $$props;
  let inputValue = "";
  if ($$props.symbol === void 0 && $$bindings.symbol && symbol !== void 0) $$bindings.symbol(symbol);
  if ($$props.maxValue === void 0 && $$bindings.maxValue && maxValue !== void 0) $$bindings.maxValue(maxValue);
  if ($$props.value === void 0 && $$bindings.value && value !== void 0) $$bindings.value(value);
  return `<div class="w-full"><div class="flex w-full"><div class="relative flex w-full"><input type="text"${add_attribute("class", `focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 ${symbol && "!rounded-none !rounded-l-lg"}`, 0)}${add_attribute("value", inputValue, 0)}> ${maxValue ? `<div class="absolute right-2 flex h-10 flex-col justify-center">${validate_component(Button, "Button").$$render(
    $$result,
    {
      color: "blue",
      class: "px-2 py-1",
      size: "xs",
      pill: true
    },
    {},
    {
      default: () => {
        return `MAX`;
      }
    }
  )}</div>` : ``}</div> ${symbol ? `${validate_component(InputAddon, "InputAddon").$$render($$result, {}, {}, {
    default: () => {
      return `<span class="whitespace-nowrap">${escape(symbol)}</span>`;
    }
  })}` : ``}</div></div>`;
});
async function vaultDeposit(raindexClient, vault, amount) {
  const chainId = get_store_value(walletConnectNetwork);
  const network = raindexClient.getNetworkByChainId(chainId);
  if (network.error) {
    throw new Error(network.error.readableMsg);
  }
  await invoke("vault_deposit", {
    depositArgs: {
      vault_id: vault.vaultId.toString(),
      token: vault.token.address,
      amount: amount.toString()
    },
    transactionArgs: {
      rpcs: network.value.rpcs,
      orderbook_address: vault.orderbook,
      derivation_index: get_store_value(ledgerWalletDerivationIndex),
      chain_id: chainId
    }
  });
}
async function vaultWithdraw(raindexClient, vault, targetAmount) {
  const chainId = get_store_value(walletConnectNetwork);
  const network = raindexClient.getNetworkByChainId(chainId);
  if (network.error) {
    throw new Error(network.error.readableMsg);
  }
  await invoke("vault_withdraw", {
    chainId,
    withdrawArgs: {
      vault_id: vault.vaultId.toString(),
      token: vault.token.address,
      target_amount: targetAmount.toString()
    },
    transactionArgs: {
      rpcs: network.value.rpcs,
      orderbook_address: vault.orderbook,
      derivation_index: get_store_value(ledgerWalletDerivationIndex),
      chain_id: chainId
    }
  });
}
const ModalVaultDeposit = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  const raindexClient = useRaindexClient();
  let { open = false } = $$props;
  let { vault } = $$props;
  let { onDeposit } = $$props;
  let amount;
  let isSubmitting = false;
  let selectWallet = false;
  let userBalance = {
    balance: Float.parse("0").value,
    formattedBalance: "0"
  };
  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = Float.parse("0").value;
      selectWallet = false;
    }
  }
  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(raindexClient, vault, amount);
      onDeposit();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const allowance = await vault.getAllowance();
      if (allowance.error) {
        throw new Error(allowance.error.readableMsg);
      }
      let allowanceFloat = Float.parse(allowance.value);
      if (allowanceFloat.error) {
        throw new Error(allowanceFloat.error.readableMsg);
      }
      if (allowanceFloat.value.lt(amount).value) {
        const calldata2 = await vault.getApprovalCalldata(amount);
        if (calldata2.error) {
          throw new Error(calldata2.error.readableMsg);
        }
        const approveTx = await ethersExecute(hexToBytes(calldata2.value), vault.token.address);
        toasts.success("Approve Transaction sent successfully!");
        await approveTx.wait(1);
      }
      const calldata = await vault.getDepositCalldata(amount);
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const depositTx = await ethersExecute(hexToBytes(calldata.value), vault.orderbook);
      toasts.success("Transaction sent successfully!");
      await depositTx.wait(1);
      onDeposit();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.vault === void 0 && $$bindings.vault && vault !== void 0) $$bindings.vault(vault);
  if ($$props.onDeposit === void 0 && $$bindings.onDeposit && onDeposit !== void 0) $$bindings.onDeposit(onDeposit);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `${!selectWallet ? `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: "Deposit to Vault",
        outsideclose: true,
        size: "sm",
        open
      },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `<div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-cwu3tu">Vault ID</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(toHex(vault.vaultId))}</p></div> <div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-i6aw0q">Token</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.token.name)}</p></div> <div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-kvz23c">Owner</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.owner)}</p></div> <div class="flex justify-between"><div class="w-1/2"><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-jsrnw">Your Balance</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(userBalance.formattedBalance)}</p></div> <div class="w-1/2"><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-4y5byx">Vault Balance</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.formattedBalance)}</p></div></div> <div class="mb-6">${validate_component(Label, "Label").$$render(
            $$result,
            {
              for: "amount",
              class: "mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
            },
            {},
            {
              default: () => {
                return `Amount`;
              }
            }
          )} ${validate_component(ButtonGroup, "ButtonGroup").$$render($$result, { class: "w-full" }, {}, {
            default: () => {
              return `${validate_component(InputTokenAmount, "InputTokenAmount").$$render(
                $$result,
                {
                  symbol: vault.token.symbol,
                  maxValue: userBalance.balance,
                  value: amount
                },
                {
                  value: ($$value) => {
                    amount = $$value;
                    $$settled = false;
                  }
                },
                {}
              )}`;
            }
          })}</div> <div class="flex w-full justify-end space-x-4">${validate_component(Button, "Button").$$render(
            $$result,
            {
              color: "alternative",
              disabled: isSubmitting
            },
            {},
            {
              default: () => {
                return `Cancel`;
              }
            }
          )} ${validate_component(Button, "Button").$$render(
            $$result,
            {
              disabled: !amount || amount.isZero().value || isSubmitting
            },
            {},
            {
              default: () => {
                return `Proceed`;
              }
            }
          )}</div>`;
        }
      }
    )}` : ``} ${validate_component(ModalExecute, "ModalExecute").$$render(
      $$result,
      {
        chainId: vault.chainId,
        onBack: () => open = true,
        title: "Deposit to Vault",
        execButtonLabel: "Deposit",
        executeLedger,
        executeWalletconnect,
        open: selectWallet,
        isSubmitting
      },
      {
        open: ($$value) => {
          selectWallet = $$value;
          $$settled = false;
        },
        isSubmitting: ($$value) => {
          isSubmitting = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const ModalVaultWithdraw = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  const raindexClient = useRaindexClient();
  let { open = false } = $$props;
  let { vault } = $$props;
  let { onWithdraw } = $$props;
  let amount = Float.parse("0").value;
  let amountGTBalance;
  let isSubmitting = false;
  let selectWallet = false;
  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = Float.parse("0").value;
      selectWallet = false;
    }
  }
  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultWithdraw(raindexClient, vault, amount);
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = await vault.getWithdrawCalldata(amount);
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const tx = await ethersExecute(hexToBytes(calldata.value), vault.orderbook);
      toasts.success("Transaction sent successfully!");
      await tx.wait(1);
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.vault === void 0 && $$bindings.vault && vault !== void 0) $$bindings.vault(vault);
  if ($$props.onWithdraw === void 0 && $$bindings.onWithdraw && onWithdraw !== void 0) $$bindings.onWithdraw(onWithdraw);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    amountGTBalance = amount ? !!amount.gt(vault.balance).value : false;
    $$rendered = `${!selectWallet ? `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: "Withdraw from Vault",
        outsideclose: !isSubmitting,
        size: "sm",
        open
      },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `<div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-cwu3tu">Vault ID</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(toHex(vault.vaultId))}</p></div> <div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-i6aw0q">Token</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.token.name)}</p></div> <div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-kvz23c">Owner</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.owner)}</p></div> <div><h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white" data-svelte-h="svelte-gbjqvt">Vault Balance</h5> <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">${escape(vault.formattedBalance)}</p></div> <div class="mb-6 w-full">${validate_component(Label, "Label").$$render(
            $$result,
            {
              for: "amount",
              class: "mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
            },
            {},
            {
              default: () => {
                return `Target Amount`;
              }
            }
          )} ${validate_component(InputTokenAmount, "InputTokenAmount").$$render(
            $$result,
            {
              symbol: vault.token.symbol,
              maxValue: vault.balance,
              value: amount
            },
            {
              value: ($$value) => {
                amount = $$value;
                $$settled = false;
              }
            },
            {}
          )} ${validate_component(Helper, "Helper").$$render($$result, { color: "red", class: "h-6 text-sm" }, {}, {
            default: () => {
              return `${amountGTBalance ? `Target amount cannot exceed available balance.` : ``}`;
            }
          })}</div> <div class="flex w-full justify-end space-x-4">${validate_component(Button, "Button").$$render($$result, { color: "alternative" }, {}, {
            default: () => {
              return `Cancel`;
            }
          })} ${validate_component(Button, "Button").$$render(
            $$result,
            {
              disabled: !amount || amount.isZero().value || amountGTBalance || isSubmitting
            },
            {},
            {
              default: () => {
                return `Proceed`;
              }
            }
          )}</div>`;
        }
      }
    )}` : ``} ${validate_component(ModalExecute, "ModalExecute").$$render(
      $$result,
      {
        chainId: vault.chainId,
        onBack: () => open = true,
        title: "Withdraw from Vault",
        execButtonLabel: "Withdraw",
        executeLedger,
        executeWalletconnect,
        open: selectWallet,
        isSubmitting
      },
      {
        open: ($$value) => {
          selectWallet = $$value;
          $$settled = false;
        },
        isSubmitting: ($$value) => {
          isSubmitting = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const ModalOrderRemove = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  const raindexClient = useRaindexClient();
  let { order } = $$props;
  let { onOrderRemoved } = $$props;
  let isSubmitting = false;
  let openOrderRemoveModal = true;
  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(raindexClient, order);
      onOrderRemoved();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = order.getRemoveCalldata();
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const tx = await ethersExecute(hexToBytes(calldata.value), order.orderbook);
      toasts.success("Transaction sent successfully!");
      await tx.wait(1);
      onOrderRemoved();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }
  if ($$props.order === void 0 && $$bindings.order && order !== void 0) $$bindings.order(order);
  if ($$props.onOrderRemoved === void 0 && $$bindings.onOrderRemoved && onOrderRemoved !== void 0) $$bindings.onOrderRemoved(onOrderRemoved);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$rendered = `${validate_component(ModalExecute, "ModalExecute").$$render(
      $$result,
      {
        chainId: order.chainId,
        title: "Remove Order",
        execButtonLabel: "Remove Order",
        executeLedger,
        executeWalletconnect,
        open: openOrderRemoveModal,
        isSubmitting
      },
      {
        open: ($$value) => {
          openOrderRemoveModal = $$value;
          $$settled = false;
        },
        isSubmitting: ($$value) => {
          isSubmitting = $$value;
          $$settled = false;
        }
      },
      {}
    )}`;
  } while (!$$settled);
  return $$rendered;
});
const tradeDebug = async (txHash, rpcUrls) => {
  return await invoke("debug_trade", {
    txHash,
    rpcs: rpcUrls
  });
};
const EvalResultsTable = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let { table } = $$props;
  if ($$props.table === void 0 && $$bindings.table && table !== void 0) $$bindings.table(table);
  return `${validate_component(Table, "Table").$$render(
    $$result,
    {
      divClass: "cursor-pointer rounded-lg overflow-hidden dark:border-none border overflow-x-scroll"
    },
    {},
    {
      default: () => {
        return `${validate_component(TableHead, "TableHead").$$render($$result, {}, {}, {
          default: () => {
            return `${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
              default: () => {
                return `Stack item`;
              }
            })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
              default: () => {
                return `Value`;
              }
            })} ${validate_component(TableHeadCell, "TableHeadCell").$$render($$result, {}, {}, {
              default: () => {
                return `Hex`;
              }
            })}`;
          }
        })} ${validate_component(TableBody, "TableBody").$$render($$result, {}, {}, {
          default: () => {
            return `${each(table.rows[0], (value, i) => {
              return `${validate_component(TableBodyRow, "TableBodyRow").$$render($$result, {}, {}, {
                default: () => {
                  return `${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { "data-testid": "debug-stack" }, {}, {
                    default: () => {
                      return `${escape(table.columnNames[i])}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { "data-testid": "debug-value" }, {}, {
                    default: () => {
                      return `${escape(isHex(value) ? formatEther(hexToBigInt(value)) : "")}`;
                    }
                  })} ${validate_component(TableBodyCell, "TableBodyCell").$$render($$result, { "data-testid": "debug-value-hex" }, {}, {
                    default: () => {
                      return `${escape(value)}`;
                    }
                  })} `;
                }
              })}`;
            })}`;
          }
        })}`;
      }
    }
  )}`;
});
const ModalTradeDebug = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let debugQuery;
  let $debugQuery, $$unsubscribe_debugQuery = noop, $$subscribe_debugQuery = () => ($$unsubscribe_debugQuery(), $$unsubscribe_debugQuery = subscribe(debugQuery, ($$value) => $debugQuery = $$value), debugQuery);
  let { open } = $$props;
  let { txHash } = $$props;
  let { rpcUrls } = $$props;
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.txHash === void 0 && $$bindings.txHash && txHash !== void 0) $$bindings.txHash(txHash);
  if ($$props.rpcUrls === void 0 && $$bindings.rpcUrls && rpcUrls !== void 0) $$bindings.rpcUrls(rpcUrls);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$subscribe_debugQuery(debugQuery = createQuery(
      {
        queryKey: [txHash + rpcUrls.join(",")],
        queryFn: () => {
          return tradeDebug(txHash, rpcUrls);
        },
        retry: 0
      },
      queryClient
    ));
    $$rendered = `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: "Debug trade",
        outsideclose: true,
        size: "lg",
        open
      },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `<div class="flex flex-col gap-y-2 text-sm"><span data-testid="modal-trade-debug-tx-hash">Trade transaction: ${escape(txHash)}</span> <span data-testid="modal-trade-debug-rpc-url">RPCs: ${escape(rpcUrls.join(", "))}</span></div> ${$debugQuery.isLoading ? `<div data-testid="modal-trade-debug-loading-message" class="flex items-center gap-x-2">${validate_component(Spinner, "Spinner").$$render($$result, { size: "4" }, {}, {})} <span data-svelte-h="svelte-13y6lfg">Replaying trade... this can take a while.</span></div>` : ``} ${$debugQuery.isError ? `${validate_component(Alert, "Alert").$$render(
            $$result,
            {
              "data-testid": "modal-trade-debug-error",
              color: "red"
            },
            {},
            {
              default: () => {
                return `${escape($debugQuery.error)}`;
              }
            }
          )}` : ``} ${$debugQuery.data ? `${validate_component(EvalResultsTable, "EvalResultsTable").$$render($$result, { table: $debugQuery.data }, {}, {})}` : ``}`;
        }
      }
    )}`;
  } while (!$$settled);
  $$unsubscribe_debugQuery();
  return $$rendered;
});
async function debugOrderQuote(order, rpcs, inputIOIndex, outputIOIndex, blockNumber) {
  return await invoke("debug_order_quote", {
    order,
    inputIoIndex: inputIOIndex,
    outputIoIndex: outputIOIndex,
    blockNumber,
    rpcs
  });
}
const ModalQuoteDebug = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let debugQuery;
  let $debugQuery, $$unsubscribe_debugQuery = noop, $$subscribe_debugQuery = () => ($$unsubscribe_debugQuery(), $$unsubscribe_debugQuery = subscribe(debugQuery, ($$value) => $debugQuery = $$value), debugQuery);
  const raindexClient = useRaindexClient();
  let { open } = $$props;
  let { order } = $$props;
  let { inputIOIndex } = $$props;
  let { outputIOIndex } = $$props;
  let { pair } = $$props;
  let { blockNumber } = $$props;
  if ($$props.open === void 0 && $$bindings.open && open !== void 0) $$bindings.open(open);
  if ($$props.order === void 0 && $$bindings.order && order !== void 0) $$bindings.order(order);
  if ($$props.inputIOIndex === void 0 && $$bindings.inputIOIndex && inputIOIndex !== void 0) $$bindings.inputIOIndex(inputIOIndex);
  if ($$props.outputIOIndex === void 0 && $$bindings.outputIOIndex && outputIOIndex !== void 0) $$bindings.outputIOIndex(outputIOIndex);
  if ($$props.pair === void 0 && $$bindings.pair && pair !== void 0) $$bindings.pair(pair);
  if ($$props.blockNumber === void 0 && $$bindings.blockNumber && blockNumber !== void 0) $$bindings.blockNumber(blockNumber);
  let $$settled;
  let $$rendered;
  let previous_head = $$result.head;
  do {
    $$settled = true;
    $$result.head = previous_head;
    $$subscribe_debugQuery(debugQuery = createQuery(
      {
        queryKey: [order + pair + blockNumber],
        queryFn: async () => {
          const network = raindexClient.getNetworkByChainId(order.chainId);
          if (network.error) {
            throw new Error(network.error.readableMsg);
          }
          const sgOrder = order.convertToSgOrder();
          if (sgOrder.error) {
            throw new Error(sgOrder.error.readableMsg);
          }
          const result = await debugOrderQuote(sgOrder.value, network.value.rpcs, inputIOIndex, outputIOIndex, blockNumber ? Number(blockNumber) : void 0);
          return result;
        },
        retry: 0,
        refetchOnWindowFocus: false,
        refetchInterval: false,
        refetchOnMount: true
      },
      queryClient
    ));
    $$rendered = `${validate_component(Modal, "Modal").$$render(
      $$result,
      {
        title: `Debugging quote for pair ${pair}`,
        outsideclose: true,
        size: "lg",
        open
      },
      {
        open: ($$value) => {
          open = $$value;
          $$settled = false;
        }
      },
      {
        default: () => {
          return `<div class="flex items-center">${$debugQuery.data ? `<div class="flex flex-col text-sm"><span class="whitespace-nowrap" data-testid="modal-quote-debug-block-number">Block: ${escape(blockNumber)}</span></div>` : ``} <div class="flex w-full items-center justify-end">${$debugQuery.isLoading || $debugQuery.isFetching ? `<span class="text-sm" data-testid="modal-quote-debug-loading-message" data-svelte-h="svelte-1apxvko">Getting quote stack...</span>` : ``} ${validate_component(Refresh, "Refresh").$$render(
            $$result,
            {
              "data-testid": "refreshButton",
              class: "ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400",
              spin: $debugQuery.isLoading || $debugQuery.isFetching
            },
            {},
            {}
          )}</div></div> ${$debugQuery.data ? `${!!$debugQuery.data[1] ? `${validate_component(Alert, "Alert").$$render(
            $$result,
            {
              "data-testid": "modal-quote-debug-error-partial",
              color: "red"
            },
            {},
            {
              default: () => {
                return `${escape($debugQuery.data[1])}`;
              }
            }
          )}` : ``} ${validate_component(EvalResultsTable, "EvalResultsTable").$$render($$result, { table: $debugQuery.data[0] }, {}, {})}` : ``} <div class="flex flex-col gap-y-2 text-sm"></div>`;
        }
      }
    )}`;
  } while (!$$settled);
  $$unsubscribe_debugQuery();
  return $$rendered;
});
const handleDepositModal = (vault, onDeposit, context) => {
  new ModalVaultDeposit({
    target: document.body,
    props: { open: true, vault, onDeposit },
    context
  });
};
const handleWithdrawModal = (vault, onWithdraw, context) => {
  new ModalVaultWithdraw({
    target: document.body,
    props: { open: true, vault, onWithdraw },
    context
  });
};
const handleOrderRemoveModal = (order, onOrderRemoved, context) => {
  new ModalOrderRemove({
    target: document.body,
    props: { order, onOrderRemoved },
    context
  });
};
const handleDebugTradeModal = (txHash, rpcUrls) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrls } });
};
const handleQuoteDebugModal = (order, inputIOIndex, outputIOIndex, pair, blockNumber) => {
  new ModalQuoteDebug({
    target: document.body,
    props: {
      open: true,
      order,
      inputIOIndex,
      outputIOIndex,
      pair,
      blockNumber
    }
  });
};
export {
  ButtonGroup as B,
  DEFAULT_PAGE_SIZE as D,
  QKEY_TOKENS as Q,
  TanstackAppTable as T,
  QKEY_ORDERS as a,
  DEFAULT_REFRESH_INTERVAL as b,
  QKEY_ORDER_TRADES_LIST as c,
  QKEY_ORDER_QUOTE as d,
  QKEY_ORDER as e,
  handleQuoteDebugModal as f,
  handleDebugTradeModal as g,
  handleOrderRemoveModal as h,
  handleDepositModal as i,
  handleWithdrawModal as j,
  QKEY_VAULTS as k,
  QKEY_VAULT_CHANGES as l,
  QKEY_VAULT as m,
  useAccount as u
};
//# sourceMappingURL=modal.js.map
