<script lang="ts">
  import { Alert } from 'flowbite-svelte';
  import { hasRequiredSettings, settingsText, settings, settingsFile } from '$lib/stores/settings';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { parseConfigSource } from '$lib/services/config';
  import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
  import { toasts } from '$lib/stores/toasts';
  import { ToastMessageType } from '$lib/typeshare/toast';

  async function apply(settingsContent: string): Promise<void> {
    try {
      settingsText.set(settingsContent);
      settings.set(await parseConfigSource(settingsContent));
    } catch (error) {
      reportErrorToSentry(error, SentrySeverityLevel.Info);
      toasts.error('Failed to apply settings', { message_type: ToastMessageType.Warning });
    }
  }

  const { debouncedFn: debouncedApply } = useDebouncedFn(apply, 1000);
  $: debouncedApply($settingsFile.text);
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
</FileTextarea>
