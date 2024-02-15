<script lang="ts">
  import { redirectIfSettingsNotDefined } from '$lib/utils/redirect';
  import {
    Button,
    Dropdown,
    DropdownItem,
    TableBodyCell,
    TableHeadCell,
  } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { vaultsList } from '$lib/stores/vaultsList';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { DotsVerticalOutline } from 'flowbite-svelte-icons';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
  import type { TokenVault } from '$lib/typeshare/vaultsList';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/utils/hash';
  import { bigintStringToHex } from '$lib/utils/hex';
  import AppTable from '$lib/components/AppTable.svelte';

  let showDepositModal = false;
  let showWithdrawModal = false;
  let showDepositGenericModal = false;
  let depositModalVault: TokenVault;
  let withdrawModalVault: TokenVault;

  redirectIfSettingsNotDefined();
</script>

<PageHeader title="Vaults">
  <svelte:fragment slot="actions">
    <Button color="green" size="xs" on:click={() => (showDepositGenericModal = true)}>Deposit</Button>
  </svelte:fragment>
</PageHeader>

<AppTable listStore={vaultsList} emptyMessage="No Vaults Found" on:clickRow={(e) => { goto(`/vaults/${e.detail.item.id}`); }}>
  <svelte:fragment slot="head">
    <TableHeadCell>Vault ID</TableHeadCell>
    <TableHeadCell>Owner</TableHeadCell>
    <TableHeadCell>Token</TableHeadCell>
    <TableHeadCell>Balance</TableHeadCell>
    <TableHeadCell>Orders</TableHeadCell>
    <TableHeadCell></TableHeadCell>
  </svelte:fragment>

  <svelte:fragment slot="bodyRow" let:item>
      <TableBodyCell tdClass="break-all px-4 py-2">{bigintStringToHex(item.vault_id)}</TableBodyCell>
      <TableBodyCell tdClass="break-all px-4 py-2 min-w-48"><Hash type={HashType.Wallet} value={item.owner.id} /></TableBodyCell>
      <TableBodyCell tdClass="break-word p-2 min-w-48">{item.token.name}</TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {item.balance_display}
        {item.token.symbol}
      </TableBodyCell>
      <TableBodyCell tdClass="break-all p-2 min-w-48">
        {#if item.orders}
          <div class="flex flex-wrap justify-start items-end">
            {#each item.orders.slice(0, 3) as order}
              <Button class="px-1 py-0 mt-1 mr-1" color="alternative" on:click={() => goto(`/orders/${order.id}`)}><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button>
            {/each}
            {#if item.orders.length > 3}...{/if}
          </div>
        {/if}
      </TableBodyCell>
      <TableBodyCell tdClass="px-0">
        {#if $walletAddressMatchesOrBlank(item.owner.id)}
          <Button color="alternative" outline={false} id={`vault-menu-${item.id}`} class="border-none px-2 mr-2" on:click={(e)=> {e.stopPropagation();}}>
            <DotsVerticalOutline class="dark:text-white"/>
          </Button>
        {/if}
      </TableBodyCell>
      {#if $walletAddressMatchesOrBlank(item.owner.id)}
        <Dropdown placement="bottom-end" triggeredBy={`#vault-menu-${item.id}`}>
          <DropdownItem on:click={(e) => {e.stopPropagation(); depositModalVault=item; showDepositModal = true;}}>Deposit</DropdownItem>
          <DropdownItem on:click={(e) => {e.stopPropagation(); withdrawModalVault=item; showWithdrawModal = true;}}>Withdraw</DropdownItem>
        </Dropdown>
      {/if}

    <ModalVaultDeposit bind:open={showDepositModal} vault={depositModalVault} />
    <ModalVaultWithdraw bind:open={showWithdrawModal} vault={withdrawModalVault} />
  </svelte:fragment>
</AppTable>

<ModalVaultDepositGeneric bind:open={showDepositGenericModal} />
