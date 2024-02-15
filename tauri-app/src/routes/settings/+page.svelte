<script lang="ts">
  import { Alert, Label, Input, Helper } from 'flowbite-svelte';
  import BadgeExternalLink from '$lib/components/BadgeExternalLink.svelte';
  import {
    rpcUrl,
    subgraphUrl,
    orderbookAddress,
    walletAddress,
    walletDerivationIndex,
    allRequiredSettingsValid,
    activeChain,
    forkBlockNumber
  } from '$lib/stores/settings';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import InputBlockNumber from '$lib/components/InputBlockNumber.svelte';
</script>

<PageHeader title="Settings" />

<div class="flex w-full justify-center">
  <div class="max-w-screen-lg">
    {#if !$allRequiredSettingsValid}
      <Alert color="red" class="m-8 text-lg">
        Please fill in all the settings to use the Orderbook.
      </Alert>
    {/if}

    <div class="mb-8">
      <Label class="bold mb-2 block text-xl">RPC URL</Label>
      <Input label="RPC URL" name="rpcUrl" required bind:value={$rpcUrl.value} />
      {#if !$rpcUrl.isValid && $rpcUrl.value.length > 0}
        <Helper class="mt-2 text-sm" color="red">Invalid URL</Helper>
      {/if}
      <Helper class="mt-2 text-sm">
        The URL of the blockchain node RPC endpoint you will use to submit Orderbook transactions.
        You can setup a hosted RPC account at <BadgeExternalLink
          href="https://infura.io"
          text="Infura"
        />
        for a reliable hosted RPC service provider. Or visit
        <BadgeExternalLink href="https://chainlist.org/" text="Chainlist" /> for find other publically
        available RPC providers.
      </Helper>
    </div>

    {#if $rpcUrl.isValid && $activeChain}
      <div class="mb-8">
        <Label class="bold mb-2 block text-xl">Chain</Label>
        <Input label="RPC URL" name="chainId" required bind:value={$activeChain.name} disabled />
        <Helper class="mt-2 text-sm">Automatically determined by your RPC URL.</Helper>
      </div>
    {/if}

    <div class="mb-8">
      <Label class="bold mb-2 block text-xl">Subgraph URL</Label>
      <Input label="Subgraph URL" name="subgraphUrl" required bind:value={$subgraphUrl.value} />
      {#if !$subgraphUrl.isValid && $subgraphUrl.value.length > 0}
        <Helper class="mt-2 text-sm" color="red">Invalid URL</Helper>
      {/if}
      <Helper class="mt-2 text-sm">
        The URL of the Subgraph you will use to query Orderbook data. Contact us for help setting up
        a Subgraph for your Orderbook deployment.
      </Helper>
    </div>

    <div class="mb-8">
      <Label class="bold mb-2 block text-xl">Orderbook Address</Label>
      <Input label="Subgraph URL" name="orderbookAddress" required bind:value={$orderbookAddress.value} />
      {#if !$orderbookAddress.isValid && $orderbookAddress.value.length > 0}
        <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
      {/if}
      <Helper class="mt-2 text-sm">
        The address of the deployed OrderbookV3 contract. Contact us for help setting up an
        Orderbook deployment.
      </Helper>
    </div>

    <div class="mb-8">
      <Label class="bold mb-2 block text-xl">Ledger Wallet</Label>
      <InputLedgerWallet
        bind:derivationIndex={$walletDerivationIndex}
        bind:walletAddress={$walletAddress.value}
      />
    </div>

    <div class="mb-8">
      <Label class="bold mb-2 block text-xl">Parser Fork Block Number</Label>
      <InputBlockNumber bind:value={$forkBlockNumber} required={false} />
      <Helper class="mt-2 text-sm">
        The block number to fork for parsing rainlang on the Add Order page. Automatically set to the latest block on app launch.
      </Helper>
    </div>
  </div>
</div>
