import type { Config } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';

export const parseConfig = async (text: string[], validate = false): Promise<Config> =>
  invoke('parse_config', { text, validate });
