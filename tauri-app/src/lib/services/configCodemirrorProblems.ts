import { ErrorCode, type Problem } from 'codemirror-rainlang';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { mergeDotrainConfigWithSettings, parseConfigSource } from './config';

export async function parseConfigSourceProblems(text: string) {
  const problems: Problem[] = [];

  try {
    await parseConfigSource(text);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export async function mergeDotrainConfigWithSettingsProblems(dotrain: string) {
  const problems: Problem[] = [];

  try {
    await mergeDotrainConfigWithSettings(dotrain);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export function convertErrorToProblem(e: unknown) {
  return {
    msg: typeof e === 'string' ? e : e instanceof Error ? e.message : 'something went wrong!',
    position: [0, 0],
    code: ErrorCode.InvalidRainDocument,
  } as Problem;
}
