<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { TokenVault as TokenVaultDetail } from '$lib/typeshare/vaultDetail';
  import type { TokenVault as TokenVaultListItem } from '$lib/typeshare/vaultsList';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import {
    vaultDeposit,
    vaultDepositApproveCalldata,
    vaultDepositCalldata,
  } from '$lib/services/vault';
  import { bigintStringToHex } from '$lib/utils/hex';
  import { orderbookAddress } from '$lib/stores/settings';
  import { checkAllowance, ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { walletBalance } from '$lib/stores/walletconnect';

  export let open = false;
  export let vault: TokenVaultDetail | TokenVaultListItem;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;

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
      await vaultDeposit(vault.vault_id, vault.token.id, amount);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if (!$orderbookAddress) throw Error('Select an orderbook to deposit');
      const allowance = await checkAllowance(vault.token.id, $orderbookAddress);
      if (allowance.lt(amount)) {
        const approveCalldata = (await vaultDepositApproveCalldata(
          vault.vault_id,
          vault.token.id,
          amount,
          allowance.toBigInt(),
        )) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, vault.token.id);
        toasts.success('Approve Transaction sent successfully!');
        await approveTx.wait(1);
      }

      const depositCalldata = (await vaultDepositCalldata(
        vault.vault_id,
        vault.token.id,
        amount,
      )) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, $orderbookAddress);
      toasts.success('Transaction sent successfully!');
      await depositTx.wait(1);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }

  let localWalletBalance: string | undefined = undefined;

  walletBalance.load().then((balance) => {
    if (balance) {
      localWalletBalance = balance;
    }
  });
</script>

{#if !selectWallet}
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
        Vault Balance
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {vault.balance_display}
      </p>
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Wallet Balance
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {localWalletBalance}
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
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button
        on:click={() => {
          selectWallet = true;
          open = false;
        }}
        disabled={!amount || amount === 0n || isSubmitting}
      >
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Deposit to Vault"
  execButtonLabel="Deposit"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
