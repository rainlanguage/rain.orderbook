import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import type {
  DeploymentCfg,
  RaindexClient,
  RaindexOrder,
  ScenarioCfg,
  DotrainGuiStateV1,
} from '@rainlanguage/orderbook';

export async function orderAdd(
  dotrain: string,
  deployment: DeploymentCfg,
  guiState?: DotrainGuiStateV1,
) {
  await invoke('order_add', {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: deployment.order.network.chainId,
    },
    gui_state: guiState,
  });
}

export async function orderRemove(raindexClient: RaindexClient, order: RaindexOrder) {
  const orderbook = raindexClient.getOrderbookByAddress(order.orderbook);
  if (orderbook.error) {
    throw new Error(orderbook.error.readableMsg);
  }

  await invoke('order_remove', {
    order,
    transactionArgs: {
      rpcs: orderbook.value.network.rpcs,
      orderbook_address: order.orderbook,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: order.chainId,
    },
    subgraphArgs: {
      url: orderbook.value.subgraph.url,
    },
  });
}

export async function orderAddCalldata(
  dotrain: string,
  deployment: DeploymentCfg,
  guiState?: DotrainGuiStateV1,
) {
  return await invoke('order_add_calldata', {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: undefined,
      chain_id: deployment.order.network.chainId,
    },
    gui_state: guiState,
  });
}

export async function orderAddComposeRainlang(
  dotrain: string,
  settings: string[],
  scenario: ScenarioCfg,
): Promise<string> {
  return await invoke('compose_from_scenario', {
    dotrain,
    settings,
    scenario,
  });
}

export async function validateSpecVersion(dotrain: string, settings: string[]): Promise<undefined> {
  return await invoke('validate_spec_version', {
    dotrain,
    settings,
  });
}
