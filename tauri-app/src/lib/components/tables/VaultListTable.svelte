<script lang="ts">
  import { Button, Dropdown, DropdownItem, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { Hash, HashType, ListViewOrderbookFilters } from '@rainlanguage/ui-components';

  import { bigintStringToHex } from '$lib/utils/hex';

  import {
    activeOrderbook,
    subgraphUrl,
    orderHash,
    accounts,
    activeAccountsItems,
    activeSubgraphs,
    settings,
    activeOrderStatus,
    hideZeroBalanceVaults,
    activeNetworkRef,
    activeOrderbookRef,
    activeAccounts,
  } from '$lib/stores/settings';

  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { vaultList } from '$lib/queries/vaultList';
  import { TanstackAppTable } from '@rainlanguage/ui-components';
  import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '@rainlanguage/ui-components';
  import { QKEY_VAULTS } from '@rainlanguage/ui-components';
  import { vaultBalanceDisplay } from '$lib/utils/vault';
  import {
    handleDepositGenericModal,
    handleDepositModal,
    handleWithdrawModal,
  } from '$lib/services/modal';

  import { get } from 'svelte/store';
  import { page } from '$app/stores';

  $: query = createInfiniteQuery({
    queryKey: [QKEY_VAULTS, $activeAccounts, $hideZeroBalanceVaults, $activeSubgraphs],
    queryFn: ({ pageParam }) => {
      return vaultList(
        $activeSubgraphs,
        Object.values(get(activeAccounts)),
        $hideZeroBalanceVaults,
        pageParam,
      );
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: DEFAULT_REFRESH_INTERVAL,
    enabled: !!$subgraphUrl,
  });

  const updateActiveNetworkAndOrderbook = (subgraphName: string) => {
    activeNetworkRef.set(subgraphName);
    activeOrderbookRef.set(subgraphName);
  };

  $: currentRoute = $page.url.pathname;
  $: isVaultsPage = currentRoute.startsWith('/vaults');
  $: isOrdersPage = currentRoute.startsWith('/orders');
</script>

{#if $query}
  <TanstackAppTable
    {query}
    emptyMessage="No Vaults Found"
    on:clickRow={(e) => {
      updateActiveNetworkAndOrderbook(e.detail.item.subgraphName);
      goto(`/vaults/${e.detail.item.vault.id}`);
    }}
  >
    <svelte:fragment slot="title">
      <div class="flex w-full justify-between py-4">
        <div class="flex items-center gap-x-6">
          <div class="text-3xl font-medium dark:text-white">Vaults</div>
          <Button
            disabled={!$activeOrderbook}
            size="sm"
            color="primary"
            data-testid="new-vault-button"
            on:click={() => {
              handleDepositGenericModal();
            }}>New vault</Button
          >
        </div>
        <div class="flex flex-col items-end gap-y-2">
          <ListViewOrderbookFilters
            {activeSubgraphs}
            {settings}
            {accounts}
            {activeAccountsItems}
            {activeOrderStatus}
            {orderHash}
            {hideZeroBalanceVaults}
            {isVaultsPage}
            {isOrdersPage}
          />
        </div>
      </div>
    </svelte:fragment>
    <svelte:fragment slot="head">
      <TableHeadCell padding="p-4">Network</TableHeadCell>
      <TableHeadCell padding="px-4 py-4">Vault ID</TableHeadCell>
      <TableHeadCell padding="px-4 py-4">Orderbook</TableHeadCell>
      <TableHeadCell padding="px-4 py-4">Owner</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Token</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Balance</TableHeadCell>
      <TableHeadCell padding="px-3 py-4">Input For</TableHeadCell>
      <TableHeadCell padding="px-3 py-4">Output For</TableHeadCell>
      <TableHeadCell padding="px-4 py-4"></TableHeadCell>
    </svelte:fragment>

    <svelte:fragment slot="bodyRow" let:item>
      <TableBodyCell tdClass="px-4 py-2" data-testid="vault-network">
        {item.subgraphName}
      </TableBodyCell>

      <TableBodyCell tdClass="break-all px-4 py-4" data-testid="vault-id"
        >{bigintStringToHex(item.vault.vaultId)}</TableBodyCell
      >
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-orderbook"
        ><Hash type={HashType.Identifier} value={item.vault.orderbook.id} /></TableBodyCell
      >
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-owner"
        ><Hash type={HashType.Wallet} value={item.vault.owner} /></TableBodyCell
      >
      <TableBodyCell tdClass="break-word p-2 min-w-48" data-testid="vault-token"
        >{item.vault.token.name}</TableBodyCell
      >
      <TableBodyCell tdClass="break-all p-2 min-w-48" data-testid="vault-balance">
        {vaultBalanceDisplay(item.vault)}
        {item.vault.token.symbol}
      </TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.vault.ordersAsInput.length > 0}
          <div data-testid="vault-order-inputs" class="flex flex-wrap items-end justify-start">
            {#each item.vault.ordersAsInput.slice(0, 3) as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color={order.active ? 'green' : 'yellow'}
                data-testid="vault-order-input"
                data-order-id={order.id}
                on:click={() => {
                  updateActiveNetworkAndOrderbook(item.subgraphName);
                  goto(`/orders/${order.id}`);
                }}
                ><Hash
                  type={HashType.Identifier}
                  value={order.orderHash}
                  copyOnClick={false}
                /></Button
              >
            {/each}
            {#if item.vault.ordersAsInput.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.vault.ordersAsOutput.length > 0}
          <div data-testid="vault-order-outputs" class="flex flex-wrap items-end justify-start">
            {#each item.vault.ordersAsOutput.slice(0, 3) as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color={order.active ? 'green' : 'yellow'}
                data-order-id={order.id}
                data-testid="vault-order-output"
                on:click={() => {
                  updateActiveNetworkAndOrderbook(item.subgraphName);
                  goto(`/orders/${order.id}`);
                }}
                ><Hash
                  type={HashType.Identifier}
                  value={order.orderHash}
                  copyOnClick={false}
                /></Button
              >
            {/each}
            {#if item.vault.ordersAsOutput.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="px-0 text-right">
        {#if $walletAddressMatchesOrBlank(item.vault.owner)}
          <Button
            color="alternative"
            outline={false}
            data-testid="vault-menu"
            id={`vault-menu-${item.vault.id}`}
            class="mr-2 border-none px-2"
            on:click={(e) => {
              e.stopPropagation();
            }}
          >
            <DotsVerticalOutline class="dark:text-white" />
          </Button>
        {/if}
      </TableBodyCell>
      {#if $walletAddressMatchesOrBlank(item.vault.owner)}
        <Dropdown
          data-testid="dropdown"
          placement="bottom-end"
          triggeredBy={`#vault-menu-${item.vault.id}`}
        >
          <DropdownItem
            data-testid="deposit-button"
            on:click={(e) => {
              e.stopPropagation();
              handleDepositModal(item.vault, $query.refetch);
            }}>Deposit</DropdownItem
          >
          <DropdownItem
            data-testid="withdraw-button"
            on:click={(e) => {
              e.stopPropagation();
              handleWithdrawModal(item.vault, $query.refetch);
            }}>Withdraw</DropdownItem
          >
        </Dropdown>
      {/if}
    </svelte:fragment>
  </TanstackAppTable>
{/if}
