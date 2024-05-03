<script lang="ts">
  import {
    Button,
    Dropdown,
    DropdownItem,
    Spinner,
    TableBodyCell,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { vaultsList } from '$lib/stores/vault';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
  import type { TokenVault } from '$lib/typeshare/vaultsList';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { bigintStringToHex } from '$lib/utils/hex';
  import AppTable from '$lib/components/AppTable.svelte';
  import {
    activeOrderbook,
    resetActiveNetworkRef,
    resetActiveOrderbookRef,
    subgraphUrl,
  } from '$lib/stores/settings';
  import ListViewOrderbookSelector from '$lib/components/ListViewOrderbookSelector.svelte';
  import { onMount } from 'svelte';

  onMount(async () => {
    if (!$activeOrderbook) {
      await resetActiveNetworkRef();
      resetActiveOrderbookRef();
    }
  });

  let showDepositModal = false;
  let showWithdrawModal = false;
  let showDepositGenericModal = false;
  let depositModalVault: TokenVault;
  let withdrawModalVault: TokenVault;

  $: $subgraphUrl, $vaultsList?.fetchFirst();
</script>

<PageHeader title="Vaults" />

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

{#if $vaultsList === undefined}
  <div class="flex h-16 w-full items-center justify-center">
    <Spinner class="h-8 w-8" color="white" />
  </div>
{:else}
  <AppTable
    listStore={$vaultsList}
    emptyMessage="No Vaults Found"
    on:clickRow={(e) => {
      goto(`/vaults/${e.detail.item.id}`);
    }}
  >
    <svelte:fragment slot="head">
      <TableHeadCell padding="px-4 py-4">Vault ID</TableHeadCell>
      <TableHeadCell padding="px-4 py-4">Owner</TableHeadCell>
      <TableHeadCell padding="px-3 py-4">Orders</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Token</TableHeadCell>
      <TableHeadCell padding="px-2 py-4">Balance</TableHeadCell>
      <TableHeadCell padding="px-4 py-4"></TableHeadCell>
    </svelte:fragment>

    <svelte:fragment slot="bodyRow" let:item>
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48">
        <Hash type={HashType.Identifier} value={bigintStringToHex(item.vault_id)} shorten={item.vault_id.length > 10}/>
      </TableBodyCell>
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"
        ><Hash type={HashType.Wallet} value={item.owner.id} /></TableBodyCell
      >
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.orders}
          <div class="flex flex-wrap items-end justify-start">
            {#each item.orders.slice(0, 3) as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color="alternative"
                on:click={() => goto(`/orders/${order.id}`)}
                ><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button
              >
            {/each}
            {#if item.orders.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="break-word p-2 min-w-48">{item.token.name}</TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {item.balance_display}
        {item.token.symbol}
      </TableBodyCell>
      <TableBodyCell tdClass="px-0 text-right">
        {#if $walletAddressMatchesOrBlank(item.owner.id)}
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
      {#if $walletAddressMatchesOrBlank(item.owner.id)}
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
  </AppTable>

  <ModalVaultDeposit bind:open={showDepositModal} vault={depositModalVault} />
  <ModalVaultWithdraw bind:open={showWithdrawModal} vault={withdrawModalVault} />
  <ModalVaultDepositGeneric bind:open={showDepositGenericModal} />
{/if}
