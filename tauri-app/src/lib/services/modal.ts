import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { RainEvalResultsTable, RaindexVault, RaindexOrder } from '@rainlanguage/orderbook';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import ModalTradeDebug from '$lib/components/modal/ModalTradeDebug.svelte';
import ModalQuoteDebug from '$lib/components/modal/ModalQuoteDebug.svelte';
import ModalScenarioDebug from '$lib/components/modal/ModalScenarioDebug.svelte';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: RaindexVault, onDeposit: () => void) => {
  new ModalVaultDeposit({ target: document.body, props: { open: true, vault, onDeposit } });
};

export const handleWithdrawModal = (vault: RaindexVault, onWithdraw: () => void) => {
  new ModalVaultWithdraw({ target: document.body, props: { open: true, vault, onWithdraw } });
};

export const handleOrderRemoveModal = (order: RaindexOrder, onOrderRemoved: () => void) => {
  new ModalOrderRemove({ target: document.body, props: { order, onOrderRemoved } });
};

export const handleDebugTradeModal = (txHash: string, rpcUrl: string) => {
  new ModalTradeDebug({ target: document.body, props: { open: true, txHash, rpcUrl } });
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
