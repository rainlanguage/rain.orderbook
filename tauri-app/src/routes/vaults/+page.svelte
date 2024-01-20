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
  import { vaults } from '$lib/stores/vault';

  function gotoVault(id: string) {
    goto(`/vaults/${id}`);
  }

  redirectIfSettingsNotDefined();
  vaults.refetch();
</script>

<Heading tag="h1" class="mb-8 text-center text-4xl font-bold">Vaults</Heading>

{#if $vaults.length === 0}
  <div class="text-center text-gray-900 dark:text-white">No Vaults found</div>
{:else}
  <Table divClass="mx-8 cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Owner</TableHeadCell>
      <TableHeadCell>Token</TableHeadCell>
      <TableHeadCell>Balance</TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $vaults as vault}
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
