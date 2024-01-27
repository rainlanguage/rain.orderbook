<script lang="ts">
  import { InputAddon, Button, Alert } from 'flowbite-svelte';
  import { InfoCircleSolid } from 'flowbite-svelte-icons';
  import { formatUnits, parseUnits } from 'viem';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';

  export let symbol: string | undefined = undefined;
  export let decimals: number = 0;
  export let maxValueRaw: bigint | undefined = undefined;
  export let value: string = '';
  export let valueRaw: bigint;

  $: maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: decimals,
    thousandsSeparator: '',
    radix: '.',
  };

  function complete({ detail }: { detail: InputMask }) {
    value = detail.value;
    try {
      valueRaw = parseUnits(detail.unmaskedValue, decimals);
      // eslint-disable-next-line no-empty
    } catch (e) {}
  }

  function fillMaxValue() {
    if (!maxValueRaw) return;

    valueRaw = maxValueRaw;
    value = formatUnits(maxValueRaw, decimals);
  }
</script>

<div class="w-full">
  <div class="relative flex w-full">
    <input
      class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"
      {value}
      use:imask={maskOptions}
      on:complete={complete}
    />

    {#if maxValueRaw}
      <div class="absolute right-20 flex h-10 flex-col justify-center">
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
