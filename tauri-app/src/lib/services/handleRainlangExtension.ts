import { RawRainlangExtension, type Problem } from 'codemirror-rainlang';
import { promiseTimeout } from '@rainlanguage/ui-components';
import { problemsCallback } from '$lib/services/langServices';
import { mergeDotrainConfigWithSettingsProblems } from '$lib/services/configCodemirrorProblems';
import type { ScenarioCfg } from '@rainlanguage/orderbook';

export function createRainlangExtension(
  bindings: Record<string, string>,
  deploymentScenario: ScenarioCfg | undefined,
) {
  return new RawRainlangExtension({
    diagnostics: async (text) => {
      let configProblems: Problem[] = [];
      let problems: Problem[] = [];
      try {
        // get problems with merging settings config with frontmatter
        configProblems = await mergeDotrainConfigWithSettingsProblems(text.text);
      } catch (e) {
        configProblems = [
          {
            msg: e as string,
            position: [0, 0],
            code: 9,
          },
        ];
      }
      try {
        // get problems with dotrain
        problems = await promiseTimeout(
          problemsCallback(text, bindings, deploymentScenario?.deployer.address),
          5000,
          'failed to parse on native parser',
        );
      } catch (e) {
        problems = [
          {
            msg: e as string,
            position: [0, 0],
            code: 9,
          },
        ];
      }
      return [...configProblems, ...problems];
    },
  });
}
