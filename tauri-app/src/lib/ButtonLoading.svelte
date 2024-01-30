<script lang="ts">
  import { Button, Spinner } from "flowbite-svelte";

  export let loading = false;
</script>

<Button disabled={loading} {...$$props} on:click>
  {#if loading}
    <Spinner class="mr-2 h-4 w-4" color="white" />
  {/if}
  <slot />
</Button>