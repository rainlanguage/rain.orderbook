<script lang="ts">
  import { Button, Modal, Label, Helper } from 'flowbite-svelte';
  import type { RaindexVault } from '@rainlanguage/orderbook';
  import { vaultWithdraw } from '$lib/services/vault';
  import { InputTokenAmount, useRaindexClient } from '@rainlanguage/ui-components';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { hexToBytes, parseUnits, toHex } from 'viem';
  import { Float } from '@rainlanguage/float';

  const raindexClient = useRaindexClient();

  export let open = false;
  export let vault: RaindexVault;
  export let onWithdraw: () => void;

  let amount: string = '0';
  let amountGTBalance: boolean;
  let isSubmitting = false;
  let selectWallet = false;

  let amountFloat = Float.parse(amount);
  let vaultBalanceFloat = Float.fromHex(vault.balance);

  $: amountGTBalance =
    amountFloat.value && vaultBalanceFloat.value
      ? !!amountFloat.value.gt(vaultBalanceFloat.value).value
      : false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = '0';
      selectWallet = false;
    }
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultWithdraw(raindexClient, vault, parseUnits(amount, Number(vault.token.decimals)));
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = await vault.getWithdrawCalldata(amount.toString());
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const tx = await ethersExecute(hexToBytes(calldata.value), vault.orderbook);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }
</script>

{#if !selectWallet}
  <Modal
    title="Withdraw from Vault"
    bind:open
    outsideclose={!isSubmitting}
    size="sm"
    on:close={reset}
  >
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault ID
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {toHex(vault.vaultId)}
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
        {vault.owner}
      </p>
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault Balance
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {vault.formattedBalance}
      </p>
    </div>

    <div class="mb-6 w-full">
      <Label
        for="amount"
        class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
      >
        Target Amount
      </Label>
      <InputTokenAmount
        bind:value={amount}
        symbol={vault.token.symbol}
        maxValue={vault.formattedBalance}
      />

      <Helper color="red" class="h-6 text-sm">
        {#if amountGTBalance}
          Target amount cannot exceed available balance.
        {/if}
      </Helper>
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset}>Cancel</Button>

      <Button
        on:click={() => {
          selectWallet = true;
          open = false;
        }}
        disabled={!amount || amount === '0' || amountGTBalance || isSubmitting}
      >
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  chainId={vault.chainId}
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Withdraw from Vault"
  execButtonLabel="Withdraw"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
