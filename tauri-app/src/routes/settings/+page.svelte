<script lang="ts">
  import { Alert, Heading, Label, Input, Helper, Button } from 'flowbite-svelte';
  import BadgeExternalLink from '$lib/BadgeExternalLink.svelte';
  import {
    rpcUrl,
    subgraphUrl,
    orderbookAddress,
    walletAddress,
    walletDerivationIndex,
    isRpcUrlValid,
    isSubgraphUrlValid,
    isOrderbookAddressValid,
    isWalletAddressValid,
    isSettingsDefinedAndValid,
  } from '$lib/stores/settings';
  import InputLedgerWallet from '$lib/InputLedgerWallet.svelte';
</script>

<Heading tag="h1" class="mb-8 text-center text-4xl font-bold">Settings</Heading>

{#if !$isSettingsDefinedAndValid}
  <Alert color="red" class="m-8 text-lg">
    Please fill in all the settings to use the Orderbook.
  </Alert>
{/if}

<div class="mb-8">
  <Label class="bold mb-2 block text-xl">RPC URL</Label>
  <Input label="RPC URL" name="rpcUrl" required bind:value={$rpcUrl} />
  {#if !$isRpcUrlValid && $rpcUrl.length > 0}
    <Helper class="mt-2 text-sm" color="red">Invalid URL</Helper>
  {/if}
  <Helper class="mt-2 text-sm">
    The URL of the blockchain node RPC endpoint you will use to submit Orderbook transactions. You
    can setup a hosted RPC account at <BadgeExternalLink href="https://infura.io" text="Infura" />
    for a reliable hosted RPC service provider. Or visit
    <BadgeExternalLink href="https://chainlist.org/" text="Chainlist" /> for find other publically available
    RPC providerss.
  </Helper>
</div>

<div class="mb-8">
  <Label class="bold mb-2 block text-xl">Subgraph URL</Label>
  <Input label="Subgraph URL" name="subgraphUrl" required bind:value={$subgraphUrl} />
  {#if !$isSubgraphUrlValid && $subgraphUrl.length > 0}
    <Helper class="mt-2 text-sm" color="red">Invalid URL</Helper>
  {/if}
  <Helper class="mt-2 text-sm">
    The URL of the Subgraph you will use to query Orderbook data. Contact us for help setting up a
    Subgraph for your Orderbook deployment.
  </Helper>
</div>

<div class="mb-8">
  <Label class="bold mb-2 block text-xl">Orderbook Address</Label>
  <Input label="Subgraph URL" name="orderbookAddress" required bind:value={$orderbookAddress} />
  {#if !$isOrderbookAddressValid && $orderbookAddress.length > 0}
    <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
  {/if}
  <Helper class="mt-2 text-sm">
    The address of the deployed OrderbookV3 contract. Contact us for help setting up an Orderbook
    deployment.
  </Helper>
</div>

<div class="mb-8">
  <Label class="bold mb-2 block text-xl">Ledger Wallet</Label>
  <InputLedgerWallet
    bind:derivationIndex={$walletDerivationIndex}
    bind:walletAddress={$walletAddress}
  />
</div>
