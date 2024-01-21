<script lang="ts">
  import { InputAddon, ButtonGroup } from 'flowbite-svelte';
  import { parseUnits } from 'viem';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { createEventDispatcher } from 'svelte';

  export let symbol: string;
  export let decimals: number;
  export let value: string = '';
  export let valueRaw: bigint;

  const maskOptions = {
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
    } catch (e) {}
  }
</script>

<div class="flex w-full">
  <input
    class="focus:border-primary-500 block w-full border-s-0 border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 first:rounded-s-lg first:border-s last:rounded-e-lg last:border-e disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400"
    {value}
    use:imask={maskOptions}
    on:complete={complete}
  />

  <InputAddon>
    {symbol}
  </InputAddon>
</div>
