import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrls, orderbookAddress, chainId, subgraphUrl } from '$lib/stores/settings';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import type { DeploymentCfg, ScenarioCfg } from '@rainlanguage/orderbook';

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

export async function orderRemove(id: string) {
  await invoke('order_remove', {
    id,
    transactionArgs: {
      rpcs: get(rpcUrls),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: get(chainId),
    },
    subgraphArgs: {
      url: get(subgraphUrl),
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

export async function orderRemoveCalldata(id: string) {
  return await invoke('order_remove_calldata', {
    id,
    subgraphArgs: {
      url: get(subgraphUrl),
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
