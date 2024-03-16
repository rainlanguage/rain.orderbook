<script lang="ts">
  import { Alert, Label, } from 'flowbite-svelte';
  import {
    hasRequiredSettings,
    settingsText,
  } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { settingsFile }from '$lib/stores/settings';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import { walletDerivationIndex, walletAddress } from '$lib/stores/wallets';

  function apply() {
    settingsText.set($settingsFile.text);
  };
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

<div class="my-8">
  <Label class="mb-2">Ledger Wallet</Label>
  <InputLedgerWallet
    bind:derivationIndex={$walletDerivationIndex}
    bind:walletAddress={$walletAddress.value}
  />
</div>