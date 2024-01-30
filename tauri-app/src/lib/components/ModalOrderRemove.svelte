<script lang="ts">
  import ModalConfirm from '$lib/components/ModalConfirm.svelte';
  import { orderRemove } from '$lib/utils/orderRemove';

  export let open = false;
  export let orderId: string;
  let isSubmitting = false;

  async function remove() {
    isSubmitting = true;
    try {
      await orderRemove(orderId);
      open = false;
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<ModalConfirm bind:open on:confirm={remove} title="Confirm remove order" confirmText="Remove" confirmColor="blue" loading={isSubmitting}>
  Are you sure you want to remove the order?
</ModalConfirm>