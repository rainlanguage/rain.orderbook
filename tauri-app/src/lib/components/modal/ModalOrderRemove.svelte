<script lang="ts">
  import ModalExecute from '$lib/components/ModalExecute.svelte';
  import { orderRemove } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import type { RaindexOrder } from '@rainlanguage/orderbook';
  import { hexToBytes } from 'viem';
  import { useRaindexClient } from '@rainlanguage/ui-components';

  const raindexClient = useRaindexClient();

  export let order: RaindexOrder;
  export let onOrderRemoved: () => void;

  let isSubmitting = false;
  let openOrderRemoveModal = true;

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(raindexClient, order);
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
