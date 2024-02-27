<script lang="ts">
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { createEventDispatcher } from 'svelte';
  import { Button, Spinner } from 'flowbite-svelte';

  export let value: number = 0;
  export let required = true;
  export let isFetching: boolean;

  const dispatch = createEventDispatcher();

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
  };

  function complete({ detail }: { detail: InputMask }) {
    value = detail.unmaskedValue.length === 0 ? 0 : parseInt(detail.unmaskedValue);
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="relative flex w-full">
    <input
      {required}
      value={value}
      type="text"
      class="block w-full disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right p-2.5 focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 bg-gray-50 text-gray-900 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 border-gray-300 dark:border-gray-600 text-sm rounded-lg"
      use:imask={maskOptions}
      on:complete={complete}
    />
    <div class="absolute right-2 flex h-10 flex-col justify-center">
      <Button
        color="blue"
        class="px-2 py-1"
        size="xs"
        pill
        on:click={() => dispatch('clickGetLatest')}
        disabled={isFetching}
      >
        {#if isFetching}
          <Spinner size="3" class="mr-2" color="white" />
        {/if}
        GET LATEST
      </Button>
    </div>
  </div>
</div>
