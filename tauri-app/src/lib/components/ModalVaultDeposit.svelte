<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { TokenVault as TokenVaultDetail } from '$lib/typeshare/vaultDetail';
  import type { TokenVault as TokenVaultListItem } from '$lib/typeshare/vaultsList';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultDeposit } from '$lib/utils/vaultDeposit';
  import { bigintStringToHex } from '$lib/utils/hex';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';

  export let open = false;
  export let vault: TokenVaultDetail | TokenVaultListItem;
  let amount: bigint;
  let isSubmitting = false;

  function reset() {
    amount = 0n;
    isSubmitting = false;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    try {
      await vaultDeposit(vault.vault_id, vault.token.id, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {bigintStringToHex(vault.vault_id)}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Token
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.token.name}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Owner
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.owner.id}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Balance
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.balance_display}
    </p>
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <ButtonGroup class="w-full">
      <InputTokenAmount
        bind:value={amount}
        symbol={vault.token.symbol}
        decimals={vault.token.decimals}
      />
    </ButtonGroup>
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <ButtonLoading on:click={execute} disabled={!amount || amount === 0n || isSubmitting} loading={isSubmitting}>
        Submit Deposit
      </ButtonLoading>
    </div>
  </svelte:fragment>
</Modal>
