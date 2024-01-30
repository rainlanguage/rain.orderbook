<script lang="ts">
  import { Button, Modal, Label, Helper } from 'flowbite-svelte';
  import type { TokenVault } from '$lib/typeshare/vaultDetail';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultWithdraw } from '$lib/utils/vaultWithdraw';
  import { toHex } from 'viem';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';

  export let open = false;
  export let vault: TokenVault;
  let amount: bigint = 0n;
  let amountGTBalance: boolean;
  let isSubmitting = false;

  $: amountGTBalance = vault !== undefined && amount > vault.balance;

  function reset() {
    amount = 0n;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    try {
      await vaultWithdraw(vault.vault.vault_id, vault.token.id, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {toHex(vault.vault.vault_id)}
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
      Target Amount
    </Label>
    <InputTokenAmount
      bind:value={amount}
      symbol={vault.token.symbol}
      decimals={vault.token.decimals}
      maxValue={vault.balance}
    />

    <Helper color="red" class="h-6 text-sm">
      {#if amountGTBalance}
        Target amount cannot exceed available balance.
      {/if}
    </Helper>
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset}>Cancel</Button>

      <ButtonLoading
        on:click={execute}
        disabled={!amount || amount === 0n || amountGTBalance || isSubmitting}
        loading={isSubmitting}
      >
        Make Withdrawal
      </ButtonLoading>
    </div>
  </svelte:fragment>
</Modal>
