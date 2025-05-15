import { writable } from 'svelte/store';
import type { ToastProps } from '@rainlanguage/ui-components';

export const toasts = writable<ToastProps[]>([]);
