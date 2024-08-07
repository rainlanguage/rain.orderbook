<script lang="ts">
  import { Button, Modal, Label, Helper } from 'flowbite-svelte';
  import type { Vault as TokenVaultDetail } from '$lib/typeshare/vaultDetail';
  import type { Vault as TokenVaultListItem } from '$lib/typeshare/vaultsList';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultWithdraw, vaultWithdrawCalldata } from '$lib/services/vault';
  import { bigintStringToHex } from '$lib/utils/hex';
  import { orderbookAddress } from '$lib/stores/settings';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatEthersTransactionError } from '$lib/utils/transaction';

  export let open = false;
  export let vault: TokenVaultDetail | TokenVaultListItem;
  let amount: bigint = 0n;
  let amountGTBalance: boolean;
  let isSubmitting = false;
  let selectWallet = false;

  $: amountGTBalance = vault !== undefined && amount > BigInt(vault.balance);

  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = 0n;
      selectWallet = false;
    }
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultWithdraw(BigInt(vault.vault_id), vault.token.id, amount);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = (await vaultWithdrawCalldata(
        BigInt(vault.vault_id),
        vault.token.id,
        amount,
      )) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress!);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
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
        {vault.owner}
      </p>
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Balance
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {vault.balance}
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
        decimals={Number(vault.token.decimals ?? 0)}
        maxValue={BigInt(vault.balance)}
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
        disabled={!amount || amount === 0n || amountGTBalance || isSubmitting}
      >
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Withdraw from Vault"
  execButtonLabel="Withdraw"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
