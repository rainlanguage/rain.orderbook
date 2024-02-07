import { writable } from "svelte/store";

export function useCachedWritable<T>(
  key: string,
  defaultValue: T,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T
) {
  const getCache = () => {
    const cached = localStorage.getItem(key);
    return cached ? deserialize(cached) : defaultValue;
  }
  const setCache = (value?: T) => {
    if(value) {
      localStorage.setItem(key, serialize(value));
    } else {
      localStorage.removeItem(key);
    }
  }

  const data = writable<T>(getCache());

  data.subscribe((value) => {
    setCache(value);
  })

  return data;
}

export const cachedWritableString = (key: string, defaultValue = '') => useCachedWritable<string>(key, defaultValue, (v) => v, (v) => v);
export const cachedWritableInt = (key: string, defaultValue = 0) => useCachedWritable<number>(key, defaultValue, (v) => v.toString(), (v) => parseInt(v));


export const useCachedWritableOptional = <T>(
  key: string,
  defaultValue: T | undefined = undefined,
  serialize: (value: T) => string,
  deserialize: (serialized: string) => T
) => useCachedWritable<T | undefined>(key, defaultValue, (v) => v ? serialize(v) : '', (v) => v ? deserialize(v) : undefined);

export const cachedWritableIntOptional = (key: string, defaultValue = undefined) => useCachedWritableOptional<number>(key, defaultValue, (v) => v.toString(), (v) => parseInt(v));
