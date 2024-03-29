<script lang="ts">
  import { Button, Input, Helper, Spinner } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { isAddress } from 'viem';
  import { toasts } from '$lib/stores/toasts';
  import { getAddressFromLedger } from '$lib/services/wallet';
  import { reportErrorToSentry } from '$lib/services/sentry';

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  export let derivationIndex: number = 0;
  export let walletAddress: string;
  let isFetchingFromLedger: boolean;

  $: isWalletAddressValid = walletAddress && walletAddress.length > 0 && isAddress(walletAddress);

  function completeDerivationIndex({ detail }: { detail: InputMask }) {
    derivationIndex = parseInt(detail.unmaskedValue);
  }

  async function getAddress() {
    isFetchingFromLedger = true;
    try {
      const res: string = await getAddressFromLedger(derivationIndex);
      walletAddress = res;
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(`Ledger error: ${e as string}`);
    }
    isFetchingFromLedger = false;
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <Input
        label="Ledger Wallet Address"
        name="walletAddress"
        required
        bind:value={walletAddress}
      />
      <div class="absolute right-2 flex h-10 flex-col justify-center">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={getAddress}
          disabled={isFetchingFromLedger}
        >
          {#if isFetchingFromLedger}
            <Spinner size="3" class="mr-2" color="white" />
          {/if}
          CONNECT
        </Button>
      </div>
    </div>
    {#if !isWalletAddressValid && walletAddress.length > 0}
      <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
    {/if}
    <Helper class="mt-2 text-sm">The address of your Ledger wallet.</Helper>
  </div>
  <div class="w-32 grow-0 break-all">
    <input
      type="text"
      class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-32 rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
      value={derivationIndex}
      use:imask={maskOptions}
      on:complete={completeDerivationIndex}
    />
    <Helper class="break-word mt-2 text-sm">Derivation Index</Helper>
  </div>
</div>
