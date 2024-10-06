import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api';
import { rpcUrl, orderbookAddress, chainId, subgraphUrl, settingsText } from '$lib/stores/settings';
import { ledgerWalletDerivationIndex } from '$lib/stores/wallets';
import type { Deployment, Scenario } from '$lib/typeshare/config';

export async function orderAdd(dotrain: string, deployment: Deployment, deploymentKey: string) {
  const filteredDotrain = await getFilteredDotrain(dotrain, deploymentKey);
  await invoke('order_add', {
    dotrain: filteredDotrain,
    deployment,
    transactionArgs: {
      rpc_url: deployment.order.network.rpc,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: deployment.order.network['chain-id'],
    },
  });
}

export async function orderRemove(id: string) {
  await invoke('order_remove', {
    id,
    transactionArgs: {
      rpc_url: get(rpcUrl),
      orderbook_address: get(orderbookAddress),
      derivation_index: get(ledgerWalletDerivationIndex),
      chain_id: get(chainId),
    },
    subgraphArgs: {
      url: get(subgraphUrl),
    },
  });
}

export async function orderAddCalldata(
  dotrain: string,
  deployment: Deployment,
  deploymentKey: string,
) {
  const filteredDotrain = await getFilteredDotrain(dotrain, deploymentKey);
  return await invoke('order_add_calldata', {
    dotrain: filteredDotrain,
    deployment,
    transactionArgs: {
      rpc_url: deployment.order.network.rpc,
      orderbook_address: deployment.order.orderbook?.address,
      derivation_index: undefined,
      chain_id: deployment.order.network['chain-id'],
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
  settings: string,
  scenario: Scenario,
): Promise<string> {
  return await invoke('compose_from_scenario', {
    dotrain,
    settings,
    scenario,
  });
}

export async function validateRaindexVersion(
  dotrain: string,
  settings: string,
): Promise<undefined> {
  return await invoke('validate_raindex_version', {
    dotrain,
    settings,
  });
}

export async function getFilteredDotrain(dotrain: string, deployment: string): Promise<string> {
  return await invoke('dotrain_filter_by_deployment', {
    dotrain,
    deployment: [deployment],
    config: get(settingsText),
    includeGui: true,
  });
}
