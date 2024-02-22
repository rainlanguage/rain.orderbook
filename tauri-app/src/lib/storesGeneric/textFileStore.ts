import { toasts } from '$lib/stores/toasts';
import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';
import { derived, get, writable } from "svelte/store";

interface TextFileData {
  text: string;
  path: string | undefined;
  isLoading: boolean;
  isSaving: boolean;
  isSavingAs: boolean;
  isEmpty: boolean;
}

export function textFileStore(name: string, extensions: string[], defaultText: string = "") {
  const text = writable<string>(defaultText);
  const path = writable<string | undefined>();
  const isLoading = writable(false);
  const isSaving = writable(false);
  const isSavingAs = writable(false);

  const { subscribe } = derived([text, path, isLoading, isSaving, isSavingAs], ([$text, $path, $isLoading, $isSaving, $isSavingAs]) => ({
    text: $text,
    path: $path,
    isLoading: $isLoading,
    isSaving: $isSaving,
    isSavingAs: $isSavingAs,
    isEmpty: $text.length === 0,
  }));

  const defaultDialogOptions = {
    filters: [{
      name,
      extensions
    }]
  }


  /// Select a text file with a dialog and load its text + path
  async function loadFile() {
    isLoading.set(true);

    try {
      const pathValue = await open({
        title: `Load ${name}`,
        multiple: false,
        ...defaultDialogOptions
      });

      if(pathValue !== null) {
        const textValue = await readTextFile(pathValue as string);

        text.set(textValue);
        path.set(pathValue as string);
      }
    } catch(e) {
      toasts.error(e as string);
    }

    isLoading.set(false);
  }

  /// Save new text to already chosen file path
  async function saveFile() {
    const pathValue = get(path);
    if(pathValue === undefined) return;

    isSaving.set(true);

    try {
      await writeTextFile(pathValue, get(text));

      path.set(pathValue as string);
      toasts.success(`Saved to ${pathValue}`, {break_text: true});
    }
    catch(e) {
      toasts.error(e as string);
    }

    isSaving.set(false);
  }

  /// Open dialog to select file path and file name and write text to new file
  async function saveFileAs() {
    isSavingAs.set(true);

    try {
      const pathValue = await save({
        title: `Save ${name} As`,
        ...defaultDialogOptions
      });

      if(pathValue !== null) {
        await writeTextFile(pathValue as string, get(text));

        path.set(pathValue as string);
        toasts.success(`Saved to ${pathValue}`, {break_text: true});
      }

    } catch(e) {
      toasts.error(e as string);
    }

    isSavingAs.set(false);
  }

  return {
    subscribe,
    set: (val: TextFileData) => text.set(val.text),
    loadFile,
    saveFile,
    saveFileAs,
  }
}