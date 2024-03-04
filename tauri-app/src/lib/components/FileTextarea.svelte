<script lang="ts">
  import type { TextFileStore } from "$lib/storesGeneric/textFileStore";
  import ButtonLoading from "./ButtonLoading.svelte";

  export let textFile: TextFileStore;
  export let title: string;
</script>

<div class="flex w-full items-end justify-between">
  <div
    class="flex-3 grow-1 overflow-hidden overflow-ellipsis text-right tracking-tight text-gray-900 dark:text-white"
  >
    {#if $textFile.path}
      {$textFile.path}
    {:else}
      {title}
    {/if}
  </div>
  <div>
    <div class="flex justify-end gap-x-2">
      {#if $textFile.path}
        <ButtonLoading
          loading={$textFile.isSaving}
          color="green"
          on:click={textFile.saveFile}>Save</ButtonLoading
        >
      {/if}
      <ButtonLoading
        loading={$textFile.isSavingAs}
        color="green"
        on:click={textFile.saveFileAs}>Save As</ButtonLoading
      >
      <ButtonLoading
        loading={$textFile.isLoading}
        color="blue"
        on:click={textFile.loadFile}>Load</ButtonLoading
      >
    </div>
  </div>
</div>

<div class="my-4 overflow-hidden rounded-lg border dark:border-none">
  <slot name="textarea" />
</div>

<div class="flex justify-end">
  <slot name="deployment" />
</div>

<div class="flex justify-end">
  <slot name="submit" />
</div>