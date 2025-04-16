import { writable } from '@square/svelte-store';
import type { Hex } from 'viem';

export const ledgerWalletAddress = writable<Hex | null>(null);
export const ledgerWalletDerivationIndex = writable<number>(0);
