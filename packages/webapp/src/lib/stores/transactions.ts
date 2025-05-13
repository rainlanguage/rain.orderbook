import { writable } from 'svelte/store';
import type { BaseTransaction } from '@rainlanguage/ui-components';

export const transactions = writable<BaseTransaction[]>([]);
