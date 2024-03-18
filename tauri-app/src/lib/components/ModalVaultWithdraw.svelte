<script lang="ts">
  import { Button, Modal, Label, Helper } from 'flowbite-svelte';
  import type { TokenVault as TokenVaultDetail } from '$lib/typeshare/vaultDetail';
  import type { TokenVault as TokenVaultListItem } from '$lib/typeshare/vaultsList';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultWithdraw, vaultWithdrawCalldata } from '$lib/services/vault';
  import { bigintStringToHex } from '$lib/utils/hex';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { orderbookAddress } from '$lib/stores/settings';
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { ledgerWalletAddress, ledgerWalletDerivationIndex } from '$lib/stores/wallets';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';

  export let open = false;
  export let vault: TokenVaultDetail | TokenVaultListItem;
  let amount: bigint = 0n;
  let amountGTBalance: boolean;
  let isSubmitting = false;
  let selectWallet = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  $: amountGTBalance = vault !== undefined && amount > BigInt(vault.balance);

  function reset() {
    amount = 0n;
    open = false;
    selectWallet = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultWithdraw(vault.vault_id, vault.token.id, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = await vaultWithdrawCalldata(vault.vault_id, vault.token.id, amount) as Uint8Array;
      const tx = await ethersExecute(calldata, $orderbookAddress!);
      toasts.success("Transaction sent successfully!");
      await tx.wait(1);
      reset();
    } catch (e) {
      toasts.error("Transaction failed!");
    }
    isSubmitting = false;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
  {#if !selectWallet}
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
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset}>Cancel</Button>

      <Button
        on:click={() => selectWallet = true}
        disabled={!amount || amount === 0n || amountGTBalance || isSubmitting}
      >
        Proceed
      </Button>
    </div>
  {:else}
    {#if !selectedLedger && !selectedWalletconnect}
      <Button color="alternative" on:click={() => selectWallet = false}>Back</Button>
      <div class="flex flex-col w-full justify-between space-y-2">
        <Button on:click={() => selectedLedger = true}>Ledger Wallet</Button>
        <Button on:click={() => selectedWalletconnect = true}>WalletConnect</Button>
      </div>
    {:else if selectedLedger}
      <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
      <InputLedgerWallet
        bind:derivationIndex={$ledgerWalletDerivationIndex}
        bind:walletAddress={$ledgerWalletAddress.value}
      />
      <ButtonLoading on:click={executeLedger} disabled={isSubmitting || !$ledgerWalletAddress || !$ledgerWalletDerivationIndex} loading={isSubmitting}>
        Withdraw
      </ButtonLoading>
    {:else if selectedWalletconnect}
      <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
      <div class="text-lg">Note that only <b>mobile</b> wallets are supported.</div>
      <div class="flex flex-col w-full justify-between space-y-2">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={() => $walletconnectModal?.open()}
        >
        {walletconnectLabel}
        </Button>
        <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$walletconnectAccount} loading={isSubmitting}>
          Withdraw
        </ButtonLoading>
      </div>
    {/if}
  {/if}
</Modal>
