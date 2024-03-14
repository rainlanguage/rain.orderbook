import { settingsText } from "$lib/stores/settings";
import type { Config } from "$lib/typeshare/config";
import type { ConfigString } from "$lib/typeshare/configString";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";

export const mergeDotrainConfigWithSettings = async (dotrain: string): Promise<ConfigString> => invoke("merge_configstrings", {dotrain, configText: get(settingsText)});

export const convertConfigstringToConfig = async (configString: ConfigString): Promise<Config> => invoke("convert_configstring_to_config", {configString});
