import { q as get_store_value, c as create_ssr_component, k as subscribe, v as validate_component, h as escape } from "./ssr.js";
import { n as globalDotrainFile, b as settingsText, B as ButtonLoading, r as reportErrorToSentry, l as SentrySeverityLevel } from "./sentry.js";
import { ErrorCode } from "codemirror-rainlang";
import { invoke } from "@tauri-apps/api";
import { debounce } from "lodash";
import { w as writable } from "./index.js";
const checkSettingsErrors = (text) => invoke("check_settings_errors", { text });
const checkDotrainWithSettingsErrors = (dotrain, settings) => invoke("check_dotrain_with_settings_errors", {
  dotrain,
  settings
});
const getDeployments = () => {
  if (!get_store_value(globalDotrainFile).text) {
    return Promise.resolve({});
  }
  return invoke("get_deployments", {
    dotrain: get_store_value(globalDotrainFile).text,
    settings: get_store_value(settingsText)
  });
};
const getScenarios = () => {
  if (!get_store_value(globalDotrainFile).text) {
    return Promise.resolve({});
  }
  return invoke("get_scenarios", {
    dotrain: get_store_value(globalDotrainFile).text,
    settings: get_store_value(settingsText)
  });
};
const FileTextarea = create_ssr_component(($$result, $$props, $$bindings, slots) => {
  let $textFile, $$unsubscribe_textFile;
  let { textFile } = $$props;
  $$unsubscribe_textFile = subscribe(textFile, (value) => $textFile = value);
  if ($$props.textFile === void 0 && $$bindings.textFile && textFile !== void 0) $$bindings.textFile(textFile);
  $$unsubscribe_textFile();
  return `<div class="flex w-full flex-col"><div class="flex items-end gap-x-4"><div class="flex-grow">${slots.alert ? slots.alert({}) : ``}</div> <div class="flex flex-col items-end justify-end gap-2"><div class="flex gap-x-2">${$textFile.path ? `${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      loading: $textFile.isSaving,
      color: "green"
    },
    {},
    {
      default: () => {
        return `Save`;
      }
    }
  )}` : ``} ${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      loading: $textFile.isSavingAs,
      color: "green"
    },
    {},
    {
      default: () => {
        return `Save As`;
      }
    }
  )} ${validate_component(ButtonLoading, "ButtonLoading").$$render(
    $$result,
    {
      loading: $textFile.isLoading,
      color: "blue"
    },
    {},
    {
      default: () => {
        return `Load`;
      }
    }
  )}</div> ${$textFile.path ? `<div class="flex w-full justify-end overflow-hidden overflow-ellipsis text-xs text-gray-500 dark:text-gray-400">${escape($textFile.path)}</div>` : ``}</div></div></div> <div class="my-4 overflow-hidden rounded-lg border dark:border-none">${slots.textarea ? slots.textarea({}) : ``}</div> <div class="my-4">${slots.additionalFields ? slots.additionalFields({}) : ``}</div> <div class="flex justify-end">${slots.submit ? slots.submit({}) : ``}</div>`;
});
async function checkConfigErrors(settings) {
  const problems = [];
  try {
    await checkSettingsErrors(settings);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }
  return problems;
}
async function checkDotrainErrors(dotrain, settings) {
  const problems = [];
  try {
    await checkDotrainWithSettingsErrors(dotrain, settings);
  } catch (e) {
    reportErrorToSentry(e, SentrySeverityLevel.Info);
    problems.push(convertErrorToProblem(e));
  }
  return problems;
}
function convertErrorToProblem(e) {
  return {
    msg: typeof e === "string" ? e : e instanceof Error ? e.message : "something went wrong!",
    position: [0, 0],
    code: ErrorCode.InvalidRainDocument
  };
}
function createDebouncedAsyncFn(fn, wait, onSuccess, onError) {
  let currentArgs;
  const executeFn = async (args) => {
    const argsNotEqual = (args2, currentArgs2) => JSON.stringify(currentArgs2) !== JSON.stringify(args2);
    try {
      const result = await fn(...args);
      if (argsNotEqual(args, currentArgs)) return;
      onSuccess(result);
    } catch (error) {
      if (argsNotEqual(args, currentArgs)) return;
      onError(error);
    }
  };
  const debouncedFn = debounce((...args) => {
    return executeFn(args);
  }, wait);
  return (...args) => {
    currentArgs = args;
    debouncedFn(...args);
  };
}
function useDebouncedFn(fn, wait) {
  const result = writable(void 0);
  const error = writable(void 0);
  const debouncedFn = createDebouncedAsyncFn(
    fn,
    wait,
    (res) => {
      result.set(res);
      error.set(void 0);
    },
    (err) => {
      error.set(err);
      result.set(void 0);
    }
  );
  return { debouncedFn, result, error };
}
export {
  FileTextarea as F,
  getScenarios as a,
  checkSettingsErrors as b,
  checkDotrainErrors as c,
  checkConfigErrors as d,
  getDeployments as g,
  useDebouncedFn as u
};
//# sourceMappingURL=asyncDebounce.js.map
