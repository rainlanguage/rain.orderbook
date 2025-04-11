import type { Readable } from 'svelte/store';
import type { RegistryManager } from '../providers/registry/RegistryManager';

export type RegistryStore = Readable<RegistryManager>;
