import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import type { DeploymentCfg, RaindexOrder, ScenarioCfg } from '@rainlanguage/orderbook';
import { getOrderbookByChainId } from '$lib/utils/getOrderbookByChainId';

export async function orderAdd(dotrain: string, deployment: DeploymentCfg) {
  await invoke('order_add', {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: deployment.order.network.chainId,
    },
  });
}

export async function orderRemove(order: RaindexOrder) {
  const orderbook = getOrderbookByChainId(order.chainId);

  await invoke('order_remove', {
    order,
    transactionArgs: {
      rpcs: orderbook.network.rpcs,
      orderbook_address: order.orderbook,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: order.id,
    },
    subgraphArgs: {
      url: orderbook.subgraph.url,
    },
  });
}

export async function orderAddCalldata(dotrain: string, deployment: DeploymentCfg) {
  return await invoke('order_add_calldata', {
    dotrain,
    deployment,
    transactionArgs: {
      rpcs: deployment.order.network.rpcs,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: undefined,
      chain_id: deployment.order.network.chainId,
    },
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
