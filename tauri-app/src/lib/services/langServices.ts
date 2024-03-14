import { invoke } from '@tauri-apps/api';
import { ErrorCode, type Problem, TextDocumentItem, Position, Hover, CompletionItem } from "codemirror-rainlang";
import { rpcUrl } from '$lib/stores/settings';
import { get } from 'svelte/store';
import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
import { settings, activeDeployment } from '$lib/stores/settings';

/**
 * Provides problems callback by invoking related tauri command
 */
export async function problemsCallback(textDocument: TextDocumentItem): Promise<Problem[]> {
  try {
    const deployment = get(activeDeployment);
    if(!deployment) throw Error("Deployment not selected");

    const scenario = get(settings).scenarios?.[deployment.scenario];
    const bindings = scenario?.bindings ? scenario.bindings : {};
    const deployer = scenario?.deployer ? get(settings).deployers?.[scenario.deployer] : undefined;

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
  const deployment = get(activeDeployment);
  if(!deployment) throw Error("Deployment not selected");

  const scenario = get(settings).scenarios?.[deployment.scenario];
  const bindings = scenario?.bindings ? scenario.bindings : {};

  return await invoke('call_lsp_hover', { textDocument, position, bindings });
}

/**
 * Provides completion callback by invoking related tauri command
 */
export async function completionCallback(textDocument: TextDocumentItem, position: Position): Promise<CompletionItem[] | null> {
  const deployment = get(activeDeployment);
  if(!deployment) throw Error("Deployment not selected");

  const scenario = get(settings).scenarios?.[deployment.scenario];
  const bindings = scenario?.bindings ? scenario.bindings : {};

  return await invoke('call_lsp_completion', { textDocument, position, bindings });
}