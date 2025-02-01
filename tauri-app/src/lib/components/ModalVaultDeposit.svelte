<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { Vault as TokenVaultDetail } from '$lib/typeshare/subgraphTypes';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import {
    vaultDeposit,
    vaultDepositApproveCalldata,
    vaultDepositCalldata,
  } from '$lib/services/vault';
  import { bigintStringToHex } from '@rainlanguage/ui-components';
  import { orderbookAddress } from '$lib/stores/settings';
  import { checkAllowance, ethersExecute, checkERC20Balance } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatUnits } from 'viem';
  import { onMount } from 'svelte';

  export let open = false;
  export let vault: TokenVaultDetail;
  export let onDeposit: () => void;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;
  let userBalance: bigint = 0n;

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
      await vaultDeposit(BigInt(vault.vaultId), vault.token.id, amount);
      onDeposit();
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
          BigInt(vault.vaultId),
          vault.token.id,
          amount,
        )) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, vault.token.id);
        toasts.success('Approve Transaction sent successfully!');
        await approveTx.wait(1);
      }

      const depositCalldata = (await vaultDepositCalldata(
        BigInt(vault.vaultId),
        vault.token.id,
        amount,
      )) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, $orderbookAddress);
      toasts.success('Transaction sent successfully!');
      await depositTx.wait(1);
      onDeposit();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }

  async function fetchUserBalance() {
    try {
      userBalance = (await checkERC20Balance(vault.token.id)).toBigInt();
    } catch (_e) {
      userBalance = 0n;
    }
  }

  onMount(() => {
    fetchUserBalance();
  });
</script>

{#if !selectWallet}
  <Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault ID
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {bigintStringToHex(vault.vaultId)}
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

    <div class="flex justify-between">
      <div class="w-1/2">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Your Balance
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {formatUnits(userBalance, Number(vault.token.decimals ?? 0))}
        </p>
      </div>
      <div class="w-1/2">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Vault Balance
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {formatUnits(BigInt(vault.balance), Number(vault.token.decimals ?? 0))}
        </p>
      </div>
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
          decimals={Number(vault.token.decimals) ?? 0}
          maxValue={userBalance}
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
