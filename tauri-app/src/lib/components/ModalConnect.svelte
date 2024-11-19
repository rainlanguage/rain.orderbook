<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import { ledgerWalletAddress } from '$lib/stores/wallets';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import IconLedger from '$lib/components/IconLedger.svelte';
  import IconWalletConnect from '$lib/components/IconWalletConnect.svelte';
  import { walletconnectAccount } from '$lib/stores/walletconnect';

  let open = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: label = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : $ledgerWalletAddress
      ? `${$ledgerWalletAddress.slice(0, 5)}...${$ledgerWalletAddress.slice(-5)}`
      : 'Connect to Wallet';

  function reset() {
    open = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }
</script>

<div class="flex w-full flex-col py-4">
  <Button color="primary" pill on:click={() => (open = true)}>{label}</Button>
</div>

<Modal title="Connect to Wallet" bind:open outsideclose size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect && !$walletconnectAccount && !$ledgerWalletAddress}
    <div class="flex justify-center space-x-4">
      <Button class="text-lg" on:click={() => (selectedLedger = true)}>
        <div class="mr-4">
          <IconLedger />
        </div>
        Ledger Wallet
      </Button>
      <Button class="text-lg" on:click={() => (selectedWalletconnect = true)}>
        <div class="mr-3">
          <IconWalletConnect />
        </div>
        WalletConnect
      </Button>
    </div>
  {:else if selectedLedger || $ledgerWalletAddress}
    <InputLedgerWallet onConnect={reset} />
    {#if !$ledgerWalletAddress}
      <div class="flex justify-between space-x-4">
        <Button color="alternative" on:click={() => (selectedLedger = false)}>Back</Button>
      </div>
    {/if}
  {:else if selectedWalletconnect || $walletconnectAccount}
    <InputWalletConnect onConnect={reset} />
    {#if !$walletconnectAccount}
      <div class="flex justify-between space-x-4">
        <Button color="alternative" on:click={() => (selectedWalletconnect = false)}>Back</Button>
      </div>
    {/if}
  {/if}
</Modal>
