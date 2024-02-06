import { darkTheme, lightTheme } from "$lib/utils/codeMirrorThemes";
import { derived, writable } from "svelte/store";

function readColorTheme() {
  const saved = localStorage.getItem('color-theme');
  if(saved) {
    return saved;
  } else if(document.body.classList.contains('dark')) {
    return 'dark';
  } else {
    return 'light';
  }
}

export const colorTheme = writable(readColorTheme());

colorTheme.subscribe((val: string) => localStorage.setItem('color-theme', val));

export const codeMirrorTheme = derived(colorTheme, ($colorTheme) => $colorTheme === 'dark' ? darkTheme : lightTheme);