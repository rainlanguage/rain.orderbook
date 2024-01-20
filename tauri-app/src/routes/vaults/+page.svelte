<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Heading,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Button,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { vaultsList } from '$lib/stores/vaultsList';

  function gotoVault(id: string) {
    goto(`/vaults/${id}`);
  }

  redirectIfSettingsNotDefined();
  vaultsList.refetch();
</script>

<Heading tag="h1" class="mb-8 text-center text-4xl font-bold">Vaults</Heading>

{#if $vaultsList.length === 0}
  <div class="text-center text-gray-900 dark:text-white">No Vaults found</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Owner</TableHeadCell>
      <TableHeadCell>Token</TableHeadCell>
      <TableHeadCell>Balance</TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $vaultsList as vault}
        <TableBodyRow on:click={() => gotoVault(vault.id)}>
          <TableBodyCell tdClass="break-all px-4 py-2">{vault.owner.id}</TableBodyCell>
          <TableBodyCell tdClass="break-word p-2"
            >{vault.token_vaults && vault.token_vaults[0].token.name}</TableBodyCell
          >
          <TableBodyCell tdClass="break-all p-2"
            >{vault.token_vaults && vault.token_vaults[0].balance_display}
            {vault.token_vaults && vault.token_vaults[0].token.symbol}</TableBodyCell
          >
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>
{/if}
