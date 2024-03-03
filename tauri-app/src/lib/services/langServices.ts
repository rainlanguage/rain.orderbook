import { invoke } from '@tauri-apps/api';
import { ErrorCode, type Problem, TextDocumentItem, Position, Hover, CompletionItem } from "codemirror-rainlang";
import { rpcUrl } from '$lib/stores/settings';
import { get } from 'svelte/store';
import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
import { deployments, activeDeploymentIndex } from '$lib/stores/settings';

/**
 * Provides problems callback by invoking related tauri command
 */
export async function problemsCallback(textDocument: TextDocumentItem): Promise<Problem[]> {
  try {
    const deployment = get(deployments)?.[get(activeDeploymentIndex)]?.[1];
    const bindings = deployment !== undefined ? deployment.scenario.bindings : {};
    const deployer = deployment !== undefined ? deployment.scenario.deployer.address : undefined;
    return await invoke('call_lsp_problems', { textDocument, rpcUrl: get(rpcUrl), blockNumber: get(forkBlockNumber).value, bindings, deployer});
  }
  catch (err) {
    return [{
      msg: typeof err === "string" ? err : err instanceof Error ? err.message : "something went wrong!",
      position: [0, 0],
      code: ErrorCode.NativeParserError
    }]
  }
}

/**
 * Provides hover callback by invoking related tauri command
 */
export async function hoverCallback(textDocument: TextDocumentItem, position: Position): Promise<Hover | null> {
  const deployment = get(deployments)?.[get(activeDeploymentIndex)]?.[1];
  const bindings = deployment !== undefined ? deployment.scenario.bindings : {};
  return await invoke('call_lsp_hover', { textDocument, position, bindings });
}

/**
 * Provides completion callback by invoking related tauri command
 */
export async function completionCallback(textDocument: TextDocumentItem, position: Position): Promise<CompletionItem[] | null> {
  const deployment = get(deployments)?.[get(activeDeploymentIndex)]?.[1];
  const bindings = deployment !== undefined ? deployment.scenario.bindings : {};
  return await invoke('call_lsp_completion', { textDocument, position, bindings });
}