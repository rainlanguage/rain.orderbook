import type { Config } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';

export const parseYaml = async (text: string[], validate = false): Promise<Config> =>
  invoke('parse_yaml', { text, validate });

export const parseDotrainAndYaml = async (
  dotrain: string,
  settings: string,
  validate = false,
): Promise<Config> => invoke('parse_dotrain_and_yaml', { dotrain, settings, validate });
