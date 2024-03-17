<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { TokenVault as TokenVaultDetail } from '$lib/typeshare/vaultDetail';
  import type { TokenVault as TokenVaultListItem } from '$lib/typeshare/vaultsList';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultDeposit, vaultDepositApproveCalldata, vaultDepositCalldata } from '$lib/services/vault';
  import { bigintStringToHex } from '$lib/utils/hex';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { orderbookAddress } from '$lib/stores/settings';
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';
  import { walletAddress, walletDerivationIndex } from '$lib/stores/wallets';
  import { checkAllowance, ethersExecute } from '$lib/services/ethersTx';

  export let open = false;
  export let vault: TokenVaultDetail | TokenVaultListItem;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  function reset() {
    amount = 0n;
    isSubmitting = false;
    open = false;
    selectWallet = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(vault.vault_id, vault.token.id, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    if (!$orderbookAddress) throw Error("Select an orderbook to deposit");
    try {
      const allowance = await checkAllowance(amount, vault.token.id, $orderbookAddress);
      if (!allowance) {
        const approveCalldata = await vaultDepositApproveCalldata(vault.vault_id, vault.token.id, amount) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, vault.token.id)
        await approveTx.wait(1);
      }

      const depositCalldata = await vaultDepositCalldata(vault.vault_id, vault.token.id, amount) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, $orderbookAddress)
      await depositTx.wait(1);

      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
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
      <Button on:click={() => selectWallet = true} disabled={!amount || amount === 0n || isSubmitting}>
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
        bind:derivationIndex={$walletDerivationIndex}
        bind:walletAddress={$walletAddress.value}
      />
      <ButtonLoading on:click={executeLedger} disabled={isSubmitting || !$walletAddress || !$walletDerivationIndex} loading={isSubmitting}>
        Deposit
      </ButtonLoading>
    {:else if selectedWalletconnect}
      <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
      <div class="text-lg">Note that WalletConnect is only supported for <b>mobile</b> wallets.</div>
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
          Deposit
        </ButtonLoading>
      </div>
    {/if}
  {/if}
</Modal>
