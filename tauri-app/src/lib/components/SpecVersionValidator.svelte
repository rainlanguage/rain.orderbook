<script lang="ts">
  import { Alert } from 'flowbite-svelte';
  import { InfoCircleOutline } from 'flowbite-svelte-icons';

  export let error: unknown;
  console.log('error', error);
</script>

{#if error && typeof error == 'string' && error.startsWith('Spec version')}
  <Alert color="red">
    <InfoCircleOutline slot="icon" class="h-5 w-5" />
    <div class="flex flex-col">
      <span>{error}</span>
      <span
        >This order may not be compatible with this version of Raindex. Head to
        <a class="underline" href="https://github.com/rainlanguage/rain.orderbook/releases"
          >Github</a
        > to find a compatible release.</span
      >
    </div>
  </Alert>
{/if}
