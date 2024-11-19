<script lang="ts">
  import { Toast } from 'flowbite-svelte';
  import IconSuccess from '$lib/components/IconSuccess.svelte';
  import IconError from '$lib/components/IconError.svelte';
  import IconWarning from '$lib/components/IconWarning.svelte';
  import IconInfo from '$lib/components/IconInfo.svelte';
  import CloseSolid from 'flowbite-svelte-icons/CloseSolid.svelte';
  import type { ToastData } from '$lib/stores/toasts';
  import { ToastMessageType } from '$lib/typeshare/toast';

  export let toast: ToastData;
  let toastColor:
    | 'none'
    | 'gray'
    | 'red'
    | 'yellow'
    | 'green'
    | 'indigo'
    | 'purple'
    | 'blue'
    | 'primary'
    | 'orange'
    | undefined;
  $: if (toast) getToastColor();

  function getToastColor() {
    if (toast.message_type === ToastMessageType.Success) {
      return 'green';
    } else if (toast.message_type === ToastMessageType.Error) {
      return 'red';
    } else if (toast.message_type === ToastMessageType.Warning) {
      return 'yellow';
    } else if (toast.message_type === ToastMessageType.Info) {
      return 'info';
    }
  }
</script>

<div class="mt-2">
  <Toast
    color={toastColor}
    contentClass="w-full text-sm font-normal flex justify-start space-x-4 items-center pr-8"
    divClass="w-full max-w-xs p-2 text-gray-500 bg-white shadow dark:text-gray-400 dark:bg-gray-800 gap-3 relative"
  >
    <svelte:fragment slot="close-button" let:close>
      <CloseSolid
        slot="close-button"
        class="absolute right-2 top-2 h-3 w-3 hover:opacity-50"
        on:click={(e) => close(e)}
      />
    </svelte:fragment>

    {#if toast.message_type === ToastMessageType.Success}
      <IconSuccess />
    {:else if toast.message_type === ToastMessageType.Error}
      <IconError />
    {:else if toast.message_type === ToastMessageType.Warning}
      <IconWarning />
    {:else if toast.message_type === ToastMessageType.Info}
      <IconInfo />
    {/if}

    <div class="max-h-48 overflow-scroll">
      {toast.text}
    </div>
  </Toast>
</div>
