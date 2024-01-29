<script lang="ts">
  import { InputAddon, Button, Alert } from 'flowbite-svelte';
  import { InfoCircleSolid } from 'flowbite-svelte-icons';
  import { formatUnits, parseUnits } from 'viem';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';

  export let symbol: string | undefined = undefined;
  export let decimals: number = 0;
  export let maxValue: bigint | undefined = undefined;
  let valueRaw: string = '';
  export let value: bigint;

  $: maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: decimals,
    thousandsSeparator: '',
    radix: '.',
  };

  function complete({ detail }: { detail: InputMask }) {
    valueRaw = detail.value;

    if (detail.unmaskedValue.length === 0) {
      value = 0n;
    } else {
      try {
        value = parseUnits(detail.unmaskedValue, decimals);
        // eslint-disable-next-line no-empty
      } catch (e) {}
    }
  }

  function fillMaxValue() {
    if (!maxValue) return;

    value = maxValue;
    valueRaw = formatUnits(maxValue, decimals);
  }
</script>

<div class="w-full">
  <div class="relative flex w-full">
    <input
      class={`focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 ${symbol && '!rounded-none !rounded-l-lg'}`}

      value={valueRaw}
      use:imask={maskOptions}
      on:complete={complete}
    />

    {#if maxValue}
      <div class="absolute right-[5.8rem] flex h-10 flex-col justify-center">
        <Button color="blue" class="px-2 py-1" size="xs" pill on:click={fillMaxValue}>MAX</Button>
      </div>
    {/if}

    {#if symbol}
      <InputAddon>
        {symbol}
      </InputAddon>
    {/if}
  </div>
  {#if decimals === 0}
    <Alert color="yellow" border class="mt-2">
      <InfoCircleSolid slot="icon" class="h-6 w-6" />
      This token does not specify a number of decimals. <br />You are inputting the raw integer
      amount with 0 decimal places.
    </Alert>
  {/if}
</div>
