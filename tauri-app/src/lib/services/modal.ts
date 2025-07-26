import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import type { RainEvalResultsTable, RaindexVault, RaindexOrder } from '@rainlanguage/orderbook';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import ModalTradeDebug from '$lib/components/modal/ModalTradeDebug.svelte';
import ModalQuoteDebug from '$lib/components/modal/ModalQuoteDebug.svelte';
import ModalScenarioDebug from '$lib/components/modal/ModalScenarioDebug.svelte';
import type { getAllContexts } from 'svelte';

export const handleDepositModal = (
  vault: RaindexVault,
  onDeposit: () => void,
  context: ReturnType<typeof getAllContexts>,
) => {
  new ModalVaultDeposit({
    target: document.body,
    props: { open: true, vault, onDeposit },
    context,
  });
};

export const handleWithdrawModal = (
  vault: RaindexVault,
  onWithdraw: () => void,
  context: ReturnType<typeof getAllContexts>,
) => {
  new ModalVaultWithdraw({
    target: document.body,
    props: { open: true, vault, onWithdraw },
    context,
  });
};

export const handleOrderRemoveModal = (
  order: RaindexOrder,
  onOrderRemoved: () => void,
  context: ReturnType<typeof getAllContexts>,
) => {
  new ModalOrderRemove({
    target: document.body,
    props: { order, onOrderRemoved },
    context,
  });
};

export const handleDebugTradeModal = (txHash: string, rpcUrls: string[]) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrls } });
};

export const handleQuoteDebugModal = (
  order: RaindexOrder,
  inputIOIndex: number,
  outputIOIndex: number,
  pair: string,
  blockNumber?: bigint,
) => {
  new ModalQuoteDebug({
    target: document.body,
    props: {
      open: true,
      order,
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
