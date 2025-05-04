import { derived } from 'svelte/store';
import { cachedWritableString } from '@rainlanguage/ui-components';

interface ValidatedSetting<T> {
  value: T;
  isValid: boolean;
}

export function validatedStringStore(
  key: string,
  defaultValue: string,
  handleIsValid: (value: string) => boolean,
) {
  const value = cachedWritableString(key, defaultValue);
  const isValid = derived(value, handleIsValid);
  const { subscribe } = derived([value, isValid], ([$value, $isValid]) => ({
    value: $value,
    isValid: $isValid,
  }));

  return {
    subscribe,
    set: (v: ValidatedSetting<string>) => value.set(v.value),
  };
}
