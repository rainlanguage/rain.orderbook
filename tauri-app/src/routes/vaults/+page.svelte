<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Button,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { vaultsList } from '$lib/stores/vaultsList';

  function gotoVault(id: string) {
    goto(`/vaults/${id}`);
  }

  redirectIfSettingsNotDefined();
  vaultsList.refetch();
</script>

<div class="flex w-full">
  <div class="flex-1"></div>
  <h1 class="flex-0 mb-8 text-4xl font-bold text-gray-900 dark:text-white">Vaults</h1>
  <div class="flex-1">
    <div class="flex justify-end space-x-2">
      <Button color="green" size="xs" on:click={toggleDepositModal}>Deposit</Button>
      <Button color="blue" size="xs" on:click={toggleWithdrawModal}>Withdraw</Button>
    </div>
  </div>
</div>

{#if $vaultsList.length === 0}
  <div class="text-center text-gray-900 dark:text-white">No Vaults found</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Vault ID</TableHeadCell>
      <TableHeadCell>Owner</TableHeadCell>
      <TableHeadCell>Token</TableHeadCell>
      <TableHeadCell>Balance</TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $vaultsList as vault}
        <TableBodyRow on:click={() => gotoVault(vault.id)}>
          <TableBodyCell tdClass="break-all px-4 py-2">{vault.vault_id}</TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2">{vault.owner.id}</TableBodyCell>
          <TableBodyCell tdClass="break-word p-2">{vault.token.name}</TableBodyCell>
          <TableBodyCell tdClass="break-all p-2">
            {vault.balance_display}
            {vault.token.symbol}
          </TableBodyCell>
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
{/if}
