import { writable } from 'svelte/store';

export function cachedWritableStore<T>(
  key: string,
  defaultValue: T,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T,
) {
  const getCache = () => {
    const cached = localStorage.getItem(key);
    return cached !== null ? deserialize(cached) : defaultValue;
  };
  const setCache = (value?: T) => {
    if (value !== undefined) {
      localStorage.setItem(key, serialize(value));
    } else {
      localStorage.removeItem(key);
    }
  };

  const data = writable<T>(getCache());

  data.subscribe((value) => {
    setCache(value);
  });

  return data;
}

export const cachedWritableString = (key: string, defaultValue = '') =>
  cachedWritableStore<string>(
    key,
    defaultValue,
    (v) => v,
    (v) => v,
  );
export const cachedWritableInt = (key: string, defaultValue = 0) =>
  cachedWritableStore<number>(
    key,
    defaultValue,
    (v) => v.toString(),
    (v) => parseInt(v),
  );

export const cachedWritableOptionalStore = <T>(
  key: string,
  defaultValue: T | undefined = undefined,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T,
) =>
  cachedWritableStore<T | undefined>(
    key,
    defaultValue,
    (v) => (v ? serialize(v) : ''),
    (v) => (v ? deserialize(v) : undefined),
  );

export const cachedWritableIntOptional = (key: string, defaultValue = undefined) =>
  cachedWritableOptionalStore<number>(
    key,
    defaultValue,
    (v) => v.toString(),
    (v) => parseInt(v),
  );
export const cachedWritableStringOptional = (key: string, defaultValue = undefined) =>
  cachedWritableOptionalStore<string>(
    key,
    defaultValue,
    (v) => v,
    (v) => v,
  );
