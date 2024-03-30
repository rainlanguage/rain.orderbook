
<script lang="ts">
  import { Button, Input, Helper, Spinner } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { isAddress } from 'viem';
  import { toasts } from '$lib/stores/toasts';
  import { getAddressFromLedger } from '$lib/services/wallet';
  import { reportErrorToSentry } from '$lib/services/sentry';

  export let address: string = "";
  export let index: number = 0;
  let isLoading: boolean;

  $: isAddressValid = address && address?.length > 0 && isAddress(address);

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  function completeDerivationIndex({ detail }: { detail: InputMask }) {
    index = parseInt(detail.unmaskedValue);
  }

  async function getAddress() {
    isLoading = true;
    try {
      const val = await getAddressFromLedger(index);
      address = val;
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(`Ledger error: ${e as string}`);
    }
    isLoading = false;
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <Input
        label="Ledger Wallet Address"
        name="walletAddress"
        required
        bind:value={address}
      />
      <div class="absolute right-2 flex h-10 flex-col justify-center">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={getAddress}
          disabled={isLoading}
        >
          {#if isLoading}
            <Spinner size="3" class="mr-2" color="white" />
          {/if}
          CONNECT
        </Button>
      </div>
    </div>
    {#if !isAddressValid && address && address?.length > 2}
      <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
    {/if}
    <Helper class="mt-2 text-sm">Address</Helper>
  </div>
  <div class="w-16 grow-0 break-all">
    <input
      type="text"
      class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
      value={index}
      use:imask={maskOptions}
      on:complete={completeDerivationIndex}
    />
    <Helper class="break-word mt-2 text-sm">Index</Helper>
  </div>
</div>