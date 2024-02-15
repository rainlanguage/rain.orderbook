<script lang="ts">
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';

  export let value: number = 0;
  export let required = true;

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
  };

  function complete({ detail }: { detail: InputMask }) {
    value = detail.unmaskedValue.length === 0 ? 0 : parseInt(detail.unmaskedValue);
  }
</script>

<input
  {required}
  value={value}
  class="block w-full disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right p-2.5 focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 bg-gray-50 text-gray-900 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 border-gray-300 dark:border-gray-600 text-sm rounded-lg"
  use:imask={maskOptions}
  on:complete={complete}
/>
