import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { SgVault } from '@rainlanguage/orderbook/js_api';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import type { SgOrder } from '@rainlanguage/orderbook/js_api';
import ModalTradeDebug from '$lib/components/modal/ModalTradeDebug.svelte';
import type { Hex } from 'viem';
import ModalQuoteDebug from '$lib/components/modal/ModalQuoteDebug.svelte';
import type { OrderRemoveModalProps } from '@rainlanguage/ui-components';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: SgVault, onDeposit: () => void) => {

  new ModalVaultDeposit({
    target: document.body,
    props: { open: true, vault, onDeposit },
  });
};

export const handleWithdrawModal = (vault: SgVault, onWithdraw: () => void) => {
  new ModalVaultWithdraw({
    target: document.body,
    props: { open: true, vault, onWithdraw },
  });
};

export const handleOrderRemoveModal = (props: OrderRemoveModalProps) => {
  new ModalOrderRemove({
    target: document.body,
    props: {
      order: props.args.order,
      onOrderRemoved: props.args.onRemove,
    },
  });
};

export const handleDebugTradeModal = (txHash: string, rpcUrl: string) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrl } });
};

export const handleQuoteDebugModal = (
  order: SgOrder,
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
