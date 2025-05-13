import { settingsText } from '$lib/stores/settings';
import type { Config } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { get } from 'svelte/store';

export const parseConfig = async (text: string, validate = false): Promise<Config> =>
  invoke('parse_configstring', { text, validate });

export const mergeDotrainConfigWithSettings = async (
  dotrain: string,
  validate = false,
): Promise<Config> =>
  invoke('merge_configstrings', { dotrain, configText: get(settingsText), validate });
