import type { Address } from "viem";

export type Token = {
    name: string;
    symbol: string;
    decimals: number;
    address: Address
}