<script lang="ts">
  import { orderHash } from '$lib/stores/settings';
  import { Label } from 'flowbite-svelte';

  export let value = $orderHash;

  function handleChange(event: Event) {
    const input = event.target as HTMLInputElement;
    value = input.value;
    $orderHash = value;
  }
</script>

<div class="flex items-center gap-x-2">
  <Label>Order Hash</Label>
  <div class="w-32">
    <input
      type="text"
      bind:value
      on:input={handleChange}
      placeholder="0x..."
      class="block w-full rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-500 dark:bg-gray-600 dark:text-white dark:placeholder-gray-400 dark:focus:border-primary-500 dark:focus:ring-primary-500 rtl:text-right"
    />
  </div>
</div>
