<script lang="ts">
  import { Alert, Spinner } from 'flowbite-svelte';
  import { settingsText, settingsFile } from '$lib/stores/settings';
  import { PageHeader } from '@rainlanguage/ui-components';
  import CodeMirrorConfigSource from '$lib/components/CodeMirrorConfigSource.svelte';
  import FileTextarea from '$lib/components/FileTextarea.svelte';
  import { useDebouncedFn } from '$lib/utils/asyncDebounce';
  import { onMount } from 'svelte';
  import { CheckOutline, CloseOutline } from 'flowbite-svelte-icons';
  import { page } from '$app/stores';
  import { applySettings, type ApplySettingsResult } from '$lib/services/applySettings';
  import { get } from 'svelte/store';

  let settingsStatus: ApplySettingsResult['settingsStatus'] = 'checking';
  let errorMessage: string | undefined = undefined;
  let height = 500;

  function updateHeight() {
    height = window.innerHeight - 320;
  }

  onMount(() => {
    updateHeight();
    window.addEventListener('resize', updateHeight);
    const fileText = get(settingsFile).text;
    if (fileText && fileText.trim() !== '') {
      handleApply(fileText);
    } else {
      settingsStatus = 'success';
      errorMessage = undefined;
    }
    return () => window.removeEventListener('resize', updateHeight);
  });

  async function handleApply(settingsContent: string): Promise<void> {
    settingsStatus = 'checking';
    errorMessage = undefined;

    const result = await applySettings(settingsContent, settingsText);

    settingsStatus = result.settingsStatus;
    if (result.errorMessage) {
      errorMessage = result.errorMessage;
    }
  }

  const { debouncedFn: debouncedHandleApply } = useDebouncedFn(handleApply, 1000);

  $: if (
    $settingsFile.text !== undefined &&
    typeof $settingsFile.text === 'string' &&
    $settingsFile.text.trim() !== ''
  ) {
    debouncedHandleApply($settingsFile.text);
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
