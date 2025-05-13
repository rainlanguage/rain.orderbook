<script lang="ts">
  import { Alert, Spinner } from 'flowbite-svelte';
  import { hasRequiredSettings, settingsText, settings, settingsFile } from '$lib/stores/settings';
  import { PageHeader } from '@rainlanguage/ui-components';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { parseConfig } from '$lib/services/config';
  import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
  import { onMount } from 'svelte';
  import { CheckOutline, CloseOutline } from 'flowbite-svelte-icons';
  import { page } from '$app/stores';

  let settingsStatus: 'idle' | 'checking' | 'success' | 'error' = 'idle';
  let errorMessage: string | undefined = undefined;
  let height = 500;

  function updateHeight() {
    height = window.innerHeight - 320;
  }

  onMount(() => {
    updateHeight();
    window.addEventListener('resize', updateHeight);
    return () => window.removeEventListener('resize', updateHeight);
  });

  async function apply(settingsContent: string): Promise<void> {
    settingsStatus = 'checking';
    try {
      settingsText.set(settingsContent);
      settings.set(await parseConfig(settingsContent));
      settingsStatus = 'success';
    } catch (error) {
      errorMessage = error as string;
      reportErrorToSentry(error, SentrySeverityLevel.Info);
      settingsStatus = 'error';
    }
  }

  const { debouncedFn: debouncedApply } = useDebouncedFn(apply, 1000);
  $: {
    debouncedApply($settingsFile.text);
    errorMessage = undefined;
    settingsStatus = 'checking';
  }
</script>

<div class="mb-4">
  <PageHeader title="Settings" pathname={$page.url.pathname} />

  <Alert color="blue" class="mb-4 mt-8 text-lg">
    Looking for some settings to get started? Check out the <a
      class="underline"
      target="_blank"
      href="https://docs.rainlang.xyz/raindex/getting-started">getting started guide</a
    >
  </Alert>

  {#await hasRequiredSettings}
    <!-- -->
  {:then val}
    {#if !val}
      <Alert color="red" class="my-8 text-lg">
        Please fill in all the settings to use the Orderbook.
      </Alert>
    {/if}
  {/await}
</div>

<FileTextarea textFile={settingsFile}>
  <svelte:fragment slot="alert">
    {#if settingsStatus === 'checking'}
      <Alert color="blue" class="flex h-10 items-center text-blue-600 dark:text-blue-400">
        <Spinner class="mr-2" size="4" />
        <span>Checking settings...</span>
      </Alert>
    {:else if settingsStatus === 'success'}
      <Alert color="green" class="flex h-10 items-center text-green-600 dark:text-green-400">
        <CheckOutline class="mr-2" size="xs" />
        <span>Settings applied successfully</span>
      </Alert>
    {:else if settingsStatus === 'error'}
      <Alert color="red" class="flex flex-col text-red-600 dark:text-red-400">
        <div class="mb-2 flex items-center">
          <CloseOutline class="mr-2" size="xs" />
          <span>Error applying settings</span>
        </div>
        <span>{errorMessage}</span>
      </Alert>
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="textarea">
    <CodeMirrorConfigSource
      bind:value={$settingsFile.text}
      styles={{ '&': { maxHeight: `${height - (errorMessage ? 35 : 0)}px`, height: '100%' } }}
    />
  </svelte:fragment>
</FileTextarea>
