<script lang="ts">
  import { Toast } from 'flowbite-svelte';
  import CheckCircleSolid from 'flowbite-svelte-icons/CheckCircleSolid.svelte';
  import CloseCircleSolid from 'flowbite-svelte-icons/CloseCircleSolid.svelte';
  import ExclamationCircleSolid from 'flowbite-svelte-icons/ExclamationCircleSolid.svelte';
  import InfoCircleSolid from 'flowbite-svelte-icons/InfoCircleSolid.svelte';
  import type { ToastData } from '$lib/stores/toasts';

  export let toast: ToastData;
</script>

<div class="mt-2">
  {#if toast.message_type === 'Success'}
    <Toast color="green">
      <CheckCircleSolid slot="icon" class="h-5 w-5" />
      <span class={toast.break_text ? "break-all" : ''}>{toast.text}</span>
    </Toast>
  {:else if toast.message_type === 'Error'}
    <Toast color="red">
      <CloseCircleSolid slot="icon" class="h-5 w-5" />
      <span class={toast.break_text ? "break-all" : ''}>{toast.text}</span>
    </Toast>
  {:else if toast.message_type === 'Warning'}
    <Toast color="yellow">
      <ExclamationCircleSolid slot="icon" class="h-5 w-5" />
      <span class={toast.break_text ? "break-all" : ''}>{toast.text}</span>
    </Toast>
  {:else}
    <Toast color="blue">
      <InfoCircleSolid slot="icon" class="h-5 w-5" />
      <span class={toast.break_text ? "break-all" : ''}>{toast.text}</span>
    </Toast>
  {/if}
</div>
