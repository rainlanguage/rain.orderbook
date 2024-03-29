<script lang="ts">
  import { Alert, Button, Modal } from 'flowbite-svelte';
  import { hasRequiredSettings, settingsText } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { settingsFile }from '$lib/stores/settings';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import { ledgerWalletDerivationIndex, ledgerWalletAddress } from '$lib/stores/wallets';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import IconLedger from '$lib/components/IconLedger.svelte';
    import IconWalletConnect from '$lib/components/IconWalletConnect.svelte';

  let open = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  function apply() {
    settingsText.set($settingsFile.text);
  };


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
    <CodeMirrorConfigSource
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
    <div class="flex justify-center space-x-4">
      <Button class="text-lg" on:click={() => selectedLedger = true}>
        <div class="mr-4">
          <IconLedger />
        </div>
        Ledger Wallet
      </Button>
      <Button class="text-lg" on:click={() => selectedWalletconnect = true}>
        <div class="mr-3">
          <IconWalletConnect />
        </div>
        WalletConnect
      </Button>
    </div>
  {:else if selectedLedger}
    <InputLedgerWallet
      bind:derivationIndex={$ledgerWalletDerivationIndex}
      bind:walletAddress={$ledgerWalletAddress.value}
    />

    <div class="flex justify-end space-x-4">
      <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
    </div>
  {:else if selectedWalletconnect}
    <InputWalletConnect />

    <div class="flex justify-end space-x-4">
      <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
    </div>
  {/if}
</Modal>