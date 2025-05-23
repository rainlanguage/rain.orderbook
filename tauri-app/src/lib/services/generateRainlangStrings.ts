import type { ScenarioCfg } from '@rainlanguage/orderbook';
import { isEmpty } from 'lodash';
import { orderAddComposeRainlang } from '$lib/services/order';
import { SentrySeverityLevel, reportErrorToSentry } from '$lib/services/sentry';

/**
 * Composes Rainlang strings for a given set of scenarios based on dotrain text and settings.
 *
 * @param dotrainText - The base dotrain text.
 * @param settingsStrings - An array of strings representing the settings configuration.
 * @param scenarios - An optional record of scenarios to generate Rainlang for.
 * @returns A Promise that resolves to a Map where keys are ScenarioCfg objects and
 *          values are the composed Rainlang strings or error messages.
 *          Returns undefined if no scenarios are provided or an unexpected error occurs.
 */
export async function generateRainlangStrings(
  dotrainText: string,
  settingsStrings: string[],
  scenarios?: Record<string, ScenarioCfg>,
): Promise<Map<ScenarioCfg, string> | undefined> {
  try {
    if (isEmpty(scenarios)) return undefined;

    const composedRainlangForScenarios: Map<ScenarioCfg, string> = new Map();

    for (const scenario of Object.values(scenarios!)) {
      try {
        const composedRainlang = await orderAddComposeRainlang(
          dotrainText,
          settingsStrings,
          scenario,
        );
        composedRainlangForScenarios.set(scenario, composedRainlang);
      } catch (e: any) {
        composedRainlangForScenarios.set(
          scenario,
          e?.toString() || 'Error composing rainlang for scenario',
        );
      }
    }
    return composedRainlangForScenarios;
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Error);
    return undefined;
  }
}
