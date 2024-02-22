<script lang="ts">
  import { Alert, } from 'flowbite-svelte';
  import {
    hasRequiredSettings,
    settingsText,
  } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorYaml from '$lib/components/CodeMirrorYaml.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { settingsFile }from '$lib/stores/settings';
  import FileTextarea from '$lib/components/FileTextarea.svelte';

  function apply() {
    settingsText.set($settingsFile.text);
  };
</script>

<PageHeader title="Settings" />

{#await hasRequiredSettings}
  <!-- -->
{:then val}
  {#if !val}
    <Alert color="red" class="my-8 text-lg">
      Please fill in all the settings to use the Orderbook.
    </Alert>
  {/if}
{/await}

<FileTextarea textFile={settingsFile} title="New Order">
  <svelte:fragment slot="textarea">
    <CodeMirrorYaml
        bind:value={$settingsFile.text}
        styles={{ '&': { minHeight: '400px' } }}
      />
  </svelte:fragment>

  <svelte:fragment slot="submit">
    <ButtonLoading
      color="green"
      disabled={$settingsFile.isEmpty}
      on:click={apply}>Apply Yaml Settings</ButtonLoading
    >
  </svelte:fragment>
</FileTextarea>
