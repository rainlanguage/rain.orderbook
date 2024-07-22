<script lang="ts">
  import { Button, Dropdown, DropdownItem, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
  import type { Vault } from '$lib/typeshare/vaultsList';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { bigintStringToHex } from '$lib/utils/hex';
  import { activeOrderbook, subgraphUrl } from '$lib/stores/settings';
  import ListViewOrderbookSelector from '$lib/components/ListViewOrderbookSelector.svelte';
  import { createInfiniteQuery } from '@tanstack/svelte-query';
  import { vaultBalanceList } from '$lib/queries/commands';
  import TanstackAppTable from '$lib/components/tables/TanstackAppTable.svelte';
  import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '$lib/queries/constants';
  import { QKEY_VAULTS } from '$lib/queries/keys';
  import { vaultBalanceDisplay } from '$lib/utils/vault';

  let showDepositModal = false;
  let showWithdrawModal = false;
  let showDepositGenericModal = false;
  let depositModalVault: Vault;
  let withdrawModalVault: Vault;

  $: query = createInfiniteQuery({
    queryKey: [QKEY_VAULTS],
    queryFn: ({ pageParam }) => {
      return vaultBalanceList($subgraphUrl, pageParam);
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: DEFAULT_REFRESH_INTERVAL,
    enabled: !!$subgraphUrl,
  });
</script>

{#if $query}
  <TanstackAppTable
    {query}
    emptyMessage="No Vaults Found"
    on:clickRow={(e) => {
      goto(`/vaults/${e.detail.item.id}`);
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
            on:click={() => (showDepositGenericModal = true)}>New vault</Button
          >
        </div>
        <div class="flex flex-col items-end gap-y-2">
          <ListViewOrderbookSelector />
        </div>
      </div>
    </svelte:fragment>
    <svelte:fragment slot="head">
      <TableHeadCell padding="px-4 py-4">Vault ID</TableHeadCell>
      <TableHeadCell padding="px-4 py-4">Owner</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Token</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Balance</TableHeadCell>
      <TableHeadCell padding="px-3 py-4">Input For</TableHeadCell>
      <TableHeadCell padding="px-3 py-4">Output For</TableHeadCell>
      <TableHeadCell padding="px-4 py-4"></TableHeadCell>
    </svelte:fragment>

    <svelte:fragment slot="bodyRow" let:item>
      <TableBodyCell tdClass="break-all px-4 py-4">{bigintStringToHex(item.vault_id)}</TableBodyCell
      >
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"
        ><Hash type={HashType.Wallet} value={item.owner} /></TableBodyCell
      >
      <TableBodyCell tdClass="break-word p-2 min-w-48">{item.token.name}</TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {vaultBalanceDisplay(item)}
        {item.token.symbol}
      </TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.orders_as_input.length > 0}
          <div class="flex flex-wrap items-end justify-start">
            {#each item.orders_as_input.slice(0, 3) as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color="alternative"
                on:click={() => goto(`/orders/${order.id}`)}
                ><Hash
                  type={HashType.Identifier}
                  value={order.order_hash}
                  copyOnClick={false}
                /></Button
              >
            {/each}
            {#if item.orders_as_input.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.orders_as_output.length > 0}
          <div class="flex flex-wrap items-end justify-start">
            {#each item.orders_as_output.slice(0, 3) as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color="alternative"
                on:click={() => goto(`/orders/${order.id}`)}
                ><Hash
                  type={HashType.Identifier}
                  value={order.order_hash}
                  copyOnClick={false}
                /></Button
              >
            {/each}
            {#if item.orders_as_output.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="px-0 text-right">
        {#if $walletAddressMatchesOrBlank(item.owner)}
          <Button
            color="alternative"
            outline={false}
            id={`vault-menu-${item.id}`}
            class="mr-2 border-none px-2"
            on:click={(e) => {
              e.stopPropagation();
            }}
          >
            <DotsVerticalOutline class="dark:text-white" />
          </Button>
        {/if}
      </TableBodyCell>
      {#if $walletAddressMatchesOrBlank(item.owner)}
        <Dropdown placement="bottom-end" triggeredBy={`#vault-menu-${item.id}`}>
          <DropdownItem
            on:click={(e) => {
              e.stopPropagation();
              depositModalVault = item;
              showDepositModal = true;
            }}>Deposit</DropdownItem
          >
          <DropdownItem
            on:click={(e) => {
              e.stopPropagation();
              withdrawModalVault = item;
              showWithdrawModal = true;
            }}>Withdraw</DropdownItem
          >
        </Dropdown>
      {/if}
    </svelte:fragment>
  </TanstackAppTable>

  <ModalVaultDeposit bind:open={showDepositModal} vault={depositModalVault} />
  <ModalVaultWithdraw bind:open={showWithdrawModal} vault={withdrawModalVault} />
  <ModalVaultDepositGeneric bind:open={showDepositGenericModal} />
{/if}
