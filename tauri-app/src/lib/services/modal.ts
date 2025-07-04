import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { RainEvalResultsTable, SgVault, SgOrder } from '@rainlanguage/orderbook';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import ModalTradeDebug from '$lib/components/modal/ModalTradeDebug.svelte';
import type { Hex } from 'viem';
import ModalQuoteDebug from '$lib/components/modal/ModalQuoteDebug.svelte';
import ModalScenarioDebug from '$lib/components/modal/ModalScenarioDebug.svelte';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: SgVault, onDeposit: () => void) => {
  new ModalVaultDeposit({ target: document.body, props: { open: true, vault, onDeposit } });
};

export const handleWithdrawModal = (vault: SgVault, onWithdraw: () => void) => {
  new ModalVaultWithdraw({ target: document.body, props: { open: true, vault, onWithdraw } });
};

export const handleOrderRemoveModal = (order: SgOrder, onOrderRemoved: () => void) => {
  new ModalOrderRemove({ target: document.body, props: { order, onOrderRemoved } });
};

export const handleDebugTradeModal = (txHash: string, rpcUrls: string[]) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrls } });
};

export const handleQuoteDebugModal = (
  order: SgOrder,
  rpcUrls: string[],
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
      rpcUrls,
      orderbook: orderbook as Hex,
      inputIOIndex,
      outputIOIndex,
      pair,
      blockNumber,
    },
  });
};

export const handleScenarioDebugModal = (pair: string, data: RainEvalResultsTable) => {
  new ModalScenarioDebug({ target: document.body, props: { open: true, pair, data } });
};
