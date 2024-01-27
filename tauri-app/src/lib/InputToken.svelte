<script lang="ts">
  import { Helper, Input } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { isAddress } from 'viem';

  export let decimals: string = '';
  export let decimalsRaw: number;
  export let address: string = '';

  $: isAddressValid = address && address.length > 0 && isAddress(address);

  const decimalsMaskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  function decimalsComplete({ detail }: { detail: InputMask }) {
    decimals = detail.value;
    if (detail.unmaskedValue.length === 0) {
      decimalsRaw = 0;
    } else {
      decimalsRaw = parseInt(detail.unmaskedValue);
    }
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <Input label="Token Address" name="address" required bind:value={address} />
    </div>

    {#if !isAddressValid && address.length > 0}
      <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
    {/if}

    <Helper class="mt-2 text-sm">Token Address</Helper>
  </div>
  <div class="w-32 grow-0 break-all">
    <input
      value={decimals}
      class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"
      use:imask={decimalsMaskOptions}
      on:complete={decimalsComplete}
    />
    <Helper class="break-word mt-2 text-sm">Decimals</Helper>
  </div>
</div>
