<script lang="ts">
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderRemove } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import type { RaindexOrder } from '@rainlanguage/orderbook';
  import { hexToBytes } from 'viem';

  let openOrderRemoveModal = true;
  export let order: RaindexOrder;
  let isSubmitting = false;
  export let onOrderRemoved: () => void;

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(order);
      onOrderRemoved();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = order.getRemoveCalldata();
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const tx = await ethersExecute(hexToBytes(calldata.value), order.orderbook);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
      onOrderRemoved();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
  }
</script>

<ModalExecute
  chainId={order.chainId}
  bind:open={openOrderRemoveModal}
  title="Remove Order"
  execButtonLabel="Remove Order"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
