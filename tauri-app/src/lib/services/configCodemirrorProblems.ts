import { ErrorCode, type Problem } from 'codemirror-rainlang';
import { reportErrorToSentry, SentrySeverityLevel } from '$lib/services/sentry';
import { checkDotrainWithSettingsErrors, checkSettingsErrors } from './config';

export async function checkConfigErrors(settings: string[]): Promise<Problem[]> {
  const problems: Problem[] = [];

  try {
    await checkSettingsErrors(settings);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }

  return problems;
}

export async function checkDotrainErrors(dotrain: string, settings: string[]) {
  const problems: Problem[] = [];

  try {
    await checkDotrainWithSettingsErrors(dotrain, settings);
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
