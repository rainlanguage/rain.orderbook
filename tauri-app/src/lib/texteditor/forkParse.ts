/* eslint-disable @typescript-eslint/no-explicit-any */
import { invoke } from '@tauri-apps/api';
import { RainDocument, type Problem, type ComposeError, ErrorCode } from "codemirror-rainlang";

/**
 * Parses a RainDocument with native parser with hardcoded entrypoints
 * @param dotrain - The RainDocument instance
 * @param forkUrl - frok url
 * @param forkBlockNumber - fork block number
 * @returns Resolves with empty array or with array of Problems if encountered an error
 */
export async function forkParseDotrain(dotrain: RainDocument, forkUrl: string, forkBlockNumber: number): Promise<Problem[]> {
  let rainlang: string;
  const frontMatter = dotrain.frontMatter;
  try {
    // set the hardcoded entrypoints
    rainlang = await dotrain.compose(["calculate-io", "handle-io"]);
  } catch(err) { // if compose fails reject with the caught error
    if ("Reject" in (err as ComposeError)) {
      return [{
        msg: (err as any).Reject,
        position: [0, 0],
        code: ErrorCode.NativeParserError
      }]
    } else {
      return (err as any).Problems
    }
  }

  try {
    // invoke tauri fork parse command
    await invoke('fork_parse', { rainlang, frontMatter, forkUrl, forkBlockNumber });
    return [];
  } catch(err) { // if the fork call fails reject with the caught errors
    return [{
      msg: err as any as string,
      position: [0, 0],
      code: ErrorCode.NativeParserError
    }]
  }
}