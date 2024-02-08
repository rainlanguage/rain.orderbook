<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Button,
    Dropdown,
    DropdownItem,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { vaultsList } from '$lib/stores/vaultsList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { DotsVerticalOutline, FileCsvOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
  import type { TokenVault } from '$lib/typeshare/vaultsList';
  import ButtonsPagination from '$lib/components/ButtonsPagination.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/utils/hash';
  import { bigintStringToHex } from '$lib/utils/hex';

  let showDepositModal = false;
  let showWithdrawModal = false;
  let showDepositGenericModal = false;
  let depositModalVault: TokenVault;
  let withdrawModalVault: TokenVault;

  redirectIfSettingsNotDefined();
  vaultsList.fetchFirst();
</script>

<PageHeader title="Vaults" />

{#if $vaultsList.currentPage.length}
  <div class="text-center text-gray-900 dark:text-white">No Vaults found</div>
{:else}
  <Table divClass="cursor-pointer" hoverable={true}>
    <TableHead>
      <TableHeadCell>Vault ID</TableHeadCell>
      <TableHeadCell>Owner</TableHeadCell>
      <TableHeadCell>Token</TableHeadCell>
      <TableHeadCell>Balance</TableHeadCell>
      <TableHeadCell></TableHeadCell>
    </TableHead>
    <TableBody>
      {#each $vaultsList.currentPage as vault}
        <TableBodyRow on:click={() => {goto(`/vaults/${vault.id}`)}}>
          <TableBodyCell tdClass="break-all px-4 py-2">{bigintStringToHex(vault.vault_id)}</TableBodyCell>
          <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"><Hash type={HashType.Wallet} value={vault.owner.id} /></TableBodyCell>
          <TableBodyCell tdClass="break-word p-2 min-w-48">{vault.token.name}</TableBodyCell>
          <TableBodyCell tdClass="break-all p-2 min-w-48">
            {vault.balance_display}
            {vault.token.symbol}
          </TableBodyCell>
          <TableBodyCell tdClass="px-0">
            {#if $walletAddressMatchesOrBlank(vault.owner.id)}
              <Button color="alternative" outline={false} id={`vault-menu-${vault.id}`} class="border-none px-2 mr-2" on:click={(e)=> {e.stopPropagation();}}>
                <DotsVerticalOutline class="dark:text-white"/>
              </Button>
            {/if}
          </TableBodyCell>
          {#if $walletAddressMatchesOrBlank(vault.owner.id)}
            <Dropdown placement="bottom-end" triggeredBy={`#vault-menu-${vault.id}`}>
              <DropdownItem on:click={(e) => {e.stopPropagation(); depositModalVault=vault; showDepositModal = true;}}>Deposit</DropdownItem>
              <DropdownItem on:click={(e) => {e.stopPropagation(); withdrawModalVault=vault; showWithdrawModal = true;}}>Withdraw</DropdownItem>
            </Dropdown>
          {/if}
        </TableBodyRow>
      {/each}
    </TableBody>
  </Table>

  <div class="flex justify-between mt-2">
    <ButtonLoading size="xs" color="blue" on:click={() => vaultsList.exportCsv()} loading={$vaultsList.isExporting}>
      <FileCsvOutline class="w-4 h-4 mr-2"/>
      Export to CSV
    </ButtonLoading>
    <ButtonsPagination index={$vaultsList.index} on:previous={vaultsList.fetchPrev} on:next={vaultsList.fetchNext} loading={$vaultsList.isFetching} />
  </div>

  <ModalVaultDeposit bind:open={showDepositModal} vault={depositModalVault} />
  <ModalVaultWithdraw bind:open={showWithdrawModal} vault={withdrawModalVault} />
{/if}

<ModalVaultDepositGeneric bind:open={showDepositGenericModal} />
