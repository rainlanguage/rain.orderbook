<script lang="ts">
  import ModalConfirm from '$lib/ModalConfirm.svelte';
  import { orderRemove } from '$lib/utils/orderRemove';
  import {Input, Helper} from 'flowbite-svelte';

  export let open = false;
  let orderId: string;
  let isSubmitting = false;

  $: orderIdValid = orderId && orderId.length > 0;

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

<ModalConfirm bind:open on:confirm={remove} title="Remove order" confirmText="Remove" confirmColor="blue" loading={isSubmitting} disabled={!orderIdValid}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Order ID
    </h5>
    <Input bind:value={orderId} required  />
    <Helper class="mt-2 text-sm">
      A hex identifier to distinguish this Order
    </Helper>
  </div>

</ModalConfirm>