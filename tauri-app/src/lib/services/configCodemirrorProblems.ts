import { ErrorCode, type Problem } from 'codemirror-rainlang';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { parseYaml, parseDotrainAndYaml } from './config';

export async function parseConfigProblems(text: string) {
  const problems: Problem[] = [];

  try {
    await parseYaml([text], true);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export async function parseDotrainAndSettingsProblems(dotrain: string, settings: string) {
  const problems: Problem[] = [];

  try {
    await parseDotrainAndYaml(dotrain, settings);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

function convertErrorToProblem(e: unknown) {
  return {
    msg: typeof e === 'string' ? e : e instanceof Error ? e.message : 'something went wrong!',
    position: [0, 0],
    code: ErrorCode.InvalidRainDocument,
  } as Problem;
}
