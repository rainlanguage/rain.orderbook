import { invoke } from '@tauri-apps/api';
import { ErrorCode, type Problem, TextDocumentItem, Position, Hover, CompletionItem } from "codemirror-rainlang";
import { forkBlockNumber, rpcUrl } from '$lib/stores/settings';
import { get } from 'svelte/store';

/**
 * Provides problems callback by invoking related tauri command
 */
export async function problemsCallback(textDocument: TextDocumentItem): Promise<Problem[]> {
  try {
    return await invoke('call_lsp_problems', { textDocument, rpcUrl: get(rpcUrl), blockNumber: get(forkBlockNumber) });
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
  return await invoke('call_lsp_hover', { textDocument, position });
}

/**
 * Provides completion callback by invoking related tauri command
 */
export async function completionCallback(textDocument: TextDocumentItem, position: Position): Promise<CompletionItem[] | null> {
  return await invoke('call_lsp_completion', { textDocument, position });
}