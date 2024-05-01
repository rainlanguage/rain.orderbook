import type { ExtAbiDecodedErrorType } from '$lib/typeshare/decodeErrors';
import { invoke } from '@tauri-apps/api';

export const decodeErrors = async (errorData: Uint8Array): Promise<ExtAbiDecodedErrorType> => invoke("decode_errors", { error_data: errorData });