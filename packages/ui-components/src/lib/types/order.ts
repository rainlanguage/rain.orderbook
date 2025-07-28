import type { NameAndDescriptionCfg } from '@rainlanguage/orderbook';

export type ValidOrderDetail = {
	details: NameAndDescriptionCfg;
	name: string;
	dotrain: string;
};

export type InvalidOrderDetail = {
	name: string;
	error: string;
};
