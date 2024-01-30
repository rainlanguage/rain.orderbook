<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { createEventDispatcher } from 'svelte';

  export let open = false;
  export let title = "Confirm";
  export let confirmText = "Confirm";
  export let confirmColor = "green";
  export let loading = false;
  export let disabled = false;

  const dispatch = createEventDispatcher();
</script>

<Modal {title} bind:open outsideclose size="sm">
  <slot></slot>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" disabled={loading} on:click={() => (open = false)}>Cancel</Button>
      <ButtonLoading color={confirmColor} on:click={() => dispatch('confirm')} disabled={loading || disabled} loading={loading}>
        {confirmText}
      </ButtonLoading>
    </div>
  </svelte:fragment>
</Modal>
