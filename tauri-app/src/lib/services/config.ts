import { settingsText } from "$lib/stores/settings";
import type { Config, ConfigSource } from "$lib/typeshare/config";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";
import { ErrorCode, type Problem } from 'codemirror-rainlang';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';

export const parseConfigSource = async (text: string): Promise<ConfigSource> => invoke("parse_configstring", {text});

export const mergeDotrainConfigWithSettings = async (dotrain: string): Promise<ConfigSource> => invoke("merge_configstrings", {dotrain, configText: get(settingsText)});

export const convertConfigstringToConfig = async (configString: ConfigSource): Promise<Config> => invoke("convert_configstring_to_config", {configString});

export async function parseConfigSourceProblems(text: string) {
  const problems: Problem[] = [];

  try {
    await parseConfigSource(text);
  } catch(e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export async function mergeDotrainConfigWithSettingsProblems(dotrain: string) {
  const problems: Problem[] = [];

  try {
    await mergeDotrainConfigWithSettings(dotrain);
  } catch(e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
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