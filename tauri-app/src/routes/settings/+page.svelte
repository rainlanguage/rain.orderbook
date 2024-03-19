<script lang="ts">
  import { Alert, Button, Modal } from 'flowbite-svelte';
  import { hasRequiredSettings, settingsText } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorConfigString from '$lib/components/CodeMirrorConfigString.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { settingsFile }from '$lib/stores/settings';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import { ledgerWalletDerivationIndex, ledgerWalletAddress } from '$lib/stores/wallets';
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';

  let open = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  function apply() {
    settingsText.set($settingsFile.text);
  };

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  function reset() {
    open = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }
</script>

<PageHeader title="Settings" />

{#await hasRequiredSettings}
  <!-- -->
{:then val}
  {#if !val}
    <Alert color="red" class="my-8 text-lg">
      Please fill in all the settings to use the Orderbook.
    </Alert>
  {/if}
{/await}

<FileTextarea textFile={settingsFile} title="Settings">
  <svelte:fragment slot="textarea">
    <CodeMirrorConfigString
        bind:value={$settingsFile.text}
        styles={{ '&': { minHeight: '400px' } }}
      />
  </svelte:fragment>

  <svelte:fragment slot="submit">
    <ButtonLoading
      color="green"
      disabled={$settingsFile.isEmpty}
      on:click={apply}>Apply Settings</ButtonLoading
    >
  </svelte:fragment>
</FileTextarea>

<div class="flex flex-col w-full w-full py-4">
  <Button color="blue" on:click={() => open = true}>Connect to Wallet</Button>
</div>

<Modal title="Connect to Wallet" bind:open={open} outsideclose size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect}
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
  {:else if selectedWalletconnect}
    <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
    <div class="flex flex-col w-full justify-between space-y-2">
      <div class="text-lg text-center">Only <b>mobile</b> wallets are supported.</div>
      <Button
        color="blue"
        class="px-2 py-1"
        size="lg"
        pill
        on:click={() => $walletconnectModal?.open()}
      >
      {walletconnectLabel}
      </Button>
    </div>
  {/if}
</Modal>