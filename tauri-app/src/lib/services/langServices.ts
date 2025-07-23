import { invoke } from '@tauri-apps/api';
import {
  ErrorCode,
  type Problem,
  TextDocumentItem,
  Position,
  Hover,
  CompletionItem,
} from 'codemirror-rainlang';
import { get } from 'svelte/store';
import { forkBlockNumber } from '$lib/stores/forkBlockNumber';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { walletConnectNetwork } from '$lib/stores/walletconnect';
import { getNetworkByChainId } from '$lib/utils/raindexClient/getNetworkByChainId';

/**
 * Provides problems callback by invoking related tauri command
 */
export async function problemsCallback(
  textDocument: TextDocumentItem,
  bindings: Record<string, string>,
  deployerAddress: string | undefined,
): Promise<Problem[]> {
  try {
    const network = getNetworkByChainId(get(walletConnectNetwork));

    return await invoke('call_lsp_problems', {
      textDocument,
      rpcs: network.rpcs,
      blockNumber: get(forkBlockNumber).value,
      bindings,
      deployer: deployerAddress,
    });
  } catch (err) {
    reportErrorToSentry(err, SentrySeverityLevel.Info);
    return [
      {
        msg:
          typeof err === 'string'
            ? err
            : err instanceof Error
              ? err.message
              : 'something went wrong!',
        position: [0, 0],
        code: ErrorCode.NativeParserError,
      },
    ];
  }
}

/**
 * Provides hover callback by invoking related tauri command
 */
export const hoverCallback = (
  textDocument: TextDocumentItem,
  position: Position,
  bindings: Record<string, string>,
): Promise<Hover | null> => invoke('call_lsp_hover', { textDocument, position, bindings });

/**
 * Provides completion callback by invoking related tauri command
 */
export const completionCallback = async (
  textDocument: TextDocumentItem,
  position: Position,
  bindings: Record<string, string>,
): Promise<CompletionItem[] | null> =>
  invoke('call_lsp_completion', { textDocument, position, bindings });
