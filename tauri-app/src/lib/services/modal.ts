import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { Vault } from '$lib/typeshare/subgraphTypes';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import type { OrderSubgraph as OrderDetailOrder } from '@rainlanguage/orderbook/js_api';
import type { OrderSubgraph as OrderListOrder } from '@rainlanguage/orderbook/js_api';
import ModalTradeDebug from '$lib/components/modal/ModalTradeDebug.svelte';
import type { Hex } from 'viem';
import ModalQuoteDebug from '$lib/components/modal/ModalQuoteDebug.svelte';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: Vault, onDeposit: () => void) => {
  new ModalVaultDeposit({ target: document.body, props: { open: true, vault, onDeposit } });
};

export const handleWithdrawModal = (vault: Vault, onWithdraw: () => void) => {
  new ModalVaultWithdraw({ target: document.body, props: { open: true, vault, onWithdraw } });
};

export const handleOrderRemoveModal = (
  order: OrderDetailOrder | OrderListOrder,
  onOrderRemoved: () => void,
) => {
  new ModalOrderRemove({ target: document.body, props: { order, onOrderRemoved } });
};

export const handleDebugTradeModal = (txHash: string, rpcUrl: string) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrl } });
};

export const handleQuoteDebugModal = (
  order: OrderDetailOrder,
  rpcUrl: string,
  orderbook: string,
  inputIOIndex: number,
  outputIOIndex: number,
  pair: string,
  blockNumber?: number,
) => {
  new ModalQuoteDebug({
    target: document.body,
    props: {
      open: true,
      order,
      rpcUrl,
      orderbook: orderbook as Hex,
      inputIOIndex,
      outputIOIndex,
      pair,
      blockNumber,
    },
  });
};
