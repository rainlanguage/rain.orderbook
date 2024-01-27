<script lang="ts">
  import { Button, Modal, Label, Spinner, Input, Helper } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';
  import { vaultDeposit } from '$lib/utils/vaultDeposit';
  import InputToken from '$lib/InputToken.svelte';

  export let open = false;

  let vaultId: bigint;
  let tokenAddress: string;
  let tokenDecimals: number;

  let amount: string = '';
  let amountRaw: bigint;
  let isSubmitting = false;

  function reset() {
    amount = '';
    amountRaw = 0n;
    isSubmitting = false;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    await vaultDeposit(vaultId, tokenAddress, amountRaw);
    reset();
    isSubmitting = false;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <Input label="Vault ID" name="vaultId" required bind:value={vaultId} />
    <Helper class="mt-2 text-sm">
      A hex identifier to distinguish this Vault from others with the same Token and Owner
    </Helper>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Token
    </h5>
    <InputToken bind:address={tokenAddress} bind:decimalsRaw={tokenDecimals} />
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <InputTokenAmount bind:value={amount} bind:valueRaw={amountRaw} decimals={tokenDecimals} />
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={execute} disabled={!amountRaw || amountRaw === 0n || isSubmitting}>
        {#if isSubmitting}
          <Spinner class="mr-2 h-4 w-4" color="white" />
        {/if}
        Submit Deposit
      </Button>
    </div>
  </svelte:fragment>
</Modal>
