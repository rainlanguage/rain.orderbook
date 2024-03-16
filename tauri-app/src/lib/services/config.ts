import { settingsText } from "$lib/stores/settings";
import type { Config } from "$lib/typeshare/config";
import type { ConfigString } from "$lib/typeshare/configString";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";
import { ErrorCode, type Problem } from 'codemirror-rainlang';

export const parseConfigString = async (text: string): Promise<ConfigString> => invoke("parse_configstring", {text});

export const mergeDotrainConfigWithSettings = async (dotrain: string): Promise<ConfigString> => invoke("merge_configstrings", {dotrain, configText: get(settingsText)});

export const convertConfigstringToConfig = async (configString: ConfigString): Promise<Config> => invoke("convert_configstring_to_config", {configString});

export async function parseConfigStringProblems(text: string) {
  const problems: Problem[] = [];

  try {
    await parseConfigString(text);
  } catch(e) {
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export async function mergeDotrainConfigWithSettingsProblems(dotrain: string) {
  const problems: Problem[] = [];

  try {
    await mergeDotrainConfigWithSettings(dotrain);
  } catch(e) {
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

function convertErrorToProblem(e: unknown) {
  return {
    msg: typeof e === "string" ? e : e instanceof Error ? e.message : "something went wrong!",
    position: [0, 0],
    code: ErrorCode.InvalidRainDocument
  } as Problem
}