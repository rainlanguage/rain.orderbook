<script lang="ts">
  import { Alert } from 'flowbite-svelte';
  import { hasRequiredSettings, settingsText } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { settingsFile } from '$lib/stores/settings';
  import FileTextarea from '$lib/components/FileTextarea.svelte';

  function apply() {
    settingsText.set($settingsFile.text);
  }
</script>

<PageHeader title="Settings" />

<Alert color="blue" class="my-8 text-lg">
  Looking for some settings to get started? Check out the <a
    class="underline"
    target="_blank"
    href="https://docs.rainlang.xyz/raindex/getting-started">getting started guide</a
  ></Alert
>

{#await hasRequiredSettings}
  <!-- -->
{:then val}
  {#if !val}
    <Alert color="red" class="my-8 text-lg">
      Please fill in all the settings to use the Orderbook.
    </Alert>
  {/if}
{/await}

<FileTextarea textFile={settingsFile} title="Settings">
  <svelte:fragment slot="textarea">
    <CodeMirrorConfigSource
      bind:value={$settingsFile.text}
      styles={{ '&': { minHeight: '400px' } }}
    />
  </svelte:fragment>

  <svelte:fragment slot="submit">
    <ButtonLoading color="green" disabled={$settingsFile.isEmpty} on:click={apply}>
      Apply Settings
      <span data-testid="button-applysettings"></span>
    </ButtonLoading>
  </svelte:fragment>
</FileTextarea>
