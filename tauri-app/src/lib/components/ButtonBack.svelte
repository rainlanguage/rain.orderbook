<script lang="ts">
  import { Button } from "flowbite-svelte";
  import ArrowLeftSolid from "flowbite-svelte-icons/ArrowLeftSolid.svelte";
</script>

<Button outline size="xs" class="w-32" color="primary" on:click={() => history.back()}>
  <ArrowLeftSolid size="xs" /><span class="ml-2">Back</span>
</Button>