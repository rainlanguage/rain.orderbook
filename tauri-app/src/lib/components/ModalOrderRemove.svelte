<script lang="ts">
  import ModalConfirm from '$lib/components/ModalConfirm.svelte';
  import type { Order } from '$lib/typeshare/order';
  import { orderRemove } from '$lib/utils/orderRemove';

  export let open = false;
  export let order: Order;
  let isSubmitting = false;

  async function remove() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
      open = false;
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<ModalConfirm bind:open on:confirm={remove} title="Confirm remove order" confirmText="Remove" confirmColor="blue" loading={isSubmitting}>
  Are you sure you want to remove the order?
</ModalConfirm>