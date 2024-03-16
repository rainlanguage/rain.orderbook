<script lang="ts">
  import { Button, Modal, Label } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultDeposit } from '$lib/services/vault';
  import InputToken from '$lib/components/InputToken.svelte';
  import InputVaultId from '$lib/components/InputVaultId.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { walletDerivationIndex, walletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { walletconnectModal, account } from '$lib/stores/settings';

  export let open = false;
  let vaultId: bigint = 0n;
  let tokenAddress: string = '';
  let tokenDecimals: number = 0;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: label = $account
    ? `${$account.slice(0, 5)}...${$account.slice(-1 * 5)}`
    : "CONNECT"

  function reset() {
    vaultId = 0n;
    tokenAddress = '';
    tokenDecimals = 0;
    amount = 0n;
    isSubmitting = false;
    open = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await await vaultDeposit(vaultId, tokenAddress, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      await await vaultDeposit(vaultId, tokenAddress, amount);
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
      <InputVaultId bind:value={vaultId} />
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Token
      </h5>
      <InputToken bind:address={tokenAddress} bind:decimals={tokenDecimals} />
    </div>

    <div class="mb-6">
      <Label
        for="amount"
        class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
      >
        Amount
      </Label>
      <InputTokenAmount bind:value={amount} decimals={tokenDecimals} />
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <ButtonLoading on:click={() => selectWallet = true} disabled={!amount || amount === 0n || isSubmitting} loading={isSubmitting}>
        Proceed
      </ButtonLoading>
    </div>
  {:else}
    {#if !selectedLedger && !selectedWalletconnect}
      <Button color="alternative" on:click={() => selectWallet = false}>Back</Button>
      <div class="mb-6">
        <ButtonLoading on:click={() => selectedLedger = true} disabled={false} loading={isSubmitting}>
          Ledger Wallet
        </ButtonLoading>
        <ButtonLoading on:click={() => selectedWalletconnect = true} disabled={false} loading={isSubmitting}>
          WalletConnect
        </ButtonLoading>
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
      <Button
        color="blue"
        class="px-2 py-1"
        size="xs"
        pill
        on:click={() => $walletconnectModal?.open()}
      >
      {label}
      </Button>
      <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$account} loading={isSubmitting}>
        Deposit
      </ButtonLoading>
    {/if}
  {/if}
</Modal>
