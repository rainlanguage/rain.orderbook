<script lang="ts">
  import { Button, Modal, Label, Spinner } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';
  import { vaultWithdraw } from '$lib/utils/vaultWithdraw';
  import InputToken from '$lib/InputToken.svelte';
  import InputVaultId from './InputVaultId.svelte';

  export let open = false;

  let vaultId: string;
  let vaultIdRaw: bigint;
  let tokenAddress: string;
  let tokenDecimals: string;
  let tokenDecimalsRaw: number = 0;
  let amount: string = '';
  let amountRaw: bigint;
  let isSubmitting = false;

  function reset() {
    vaultId = '';
    vaultIdRaw = 0n;
    tokenAddress = '';
    tokenDecimals = '';
    tokenDecimalsRaw = 0;
    amount = '';
    amountRaw = 0n;
    isSubmitting = false;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    await vaultWithdraw(vaultIdRaw, tokenAddress, amountRaw);
    reset();
    isSubmitting = false;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <InputVaultId bind:value={vaultId} bind:valueRaw={vaultIdRaw} />
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Token
    </h5>
    <InputToken
      bind:address={tokenAddress}
      bind:decimals={tokenDecimals}
      bind:decimalsRaw={tokenDecimalsRaw}
    />
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <InputTokenAmount bind:value={amount} bind:valueRaw={amountRaw} decimals={tokenDecimalsRaw} />
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={execute} disabled={!amountRaw || amountRaw === 0n || isSubmitting}>
        {#if isSubmitting}
          <Spinner class="mr-2 h-4 w-4" color="white" />
        {/if}
        Make Withdrawal
      </Button>
    </div>
  </svelte:fragment>
</Modal>
