/* eslint-disable @typescript-eslint/no-explicit-any */
import { invoke } from '@tauri-apps/api';
import { RainDocument, ErrorCode, type Problem } from "codemirror-rainlang";

/**
 * Parses a RainDocument with native parser with hardcoded entrypoints
 * @param dotrain - The RainDocument instance
 * @param forkUrl - frok url
 * @param forkBlockNumber - fork block number
 * @returns Resolves with empty array or with array of Problems if encountered an error
 */
export async function parseDotrain(dotrain: RainDocument, forkUrl: string, forkBlockNumber: number): Promise<Problem[]> {
  let rainlang: string;
  const frontMatter = dotrain.frontMatter;
  try {
    // set the hardcoded entrypoints
    rainlang = await dotrain.compose(["calculate-io", "handle-io"]);
  } catch(err) {
    // if compose fails, reject with the caught error
    if (err && typeof err === "object") {
      if ("Reject" in err) {
        return [{
          msg: err.Reject as string,
          position: [0, 0],
          code: ErrorCode.NativeParserError
        }]
      } else if ("Problems" in err) {
        return err.Problems as Problem[]
      } else {
        // in case of unexpected panic with uknown error type
        return [{
          msg: "something went wrong: " + (typeof err === "string" ? err : err instanceof Error ? err.message : ""),
          position: [0, 0],
          code: ErrorCode.NativeParserError
        }]
      }
    } else {
      // in case of unexpected panic with uknown error type
      return [{
        msg: "something went wrong: " + (typeof err === "string" ? err : ""),
        position: [0, 0],
        code: ErrorCode.NativeParserError
      }]
    }
  }

  try {
    // invoke tauri fork parse command
    await invoke('fork_parse', { rainlang, frontMatter, forkUrl, forkBlockNumber });
    return [];
  } catch(err) {
    // if the fork call fails, reject with the caught errors
    return [{
      msg: typeof err === "string" ? err : err instanceof Error ? err.message : "",
      position: [0, 0], // default position for native parser errors without knowing offset
      code: ErrorCode.NativeParserError
    }]
  }
}