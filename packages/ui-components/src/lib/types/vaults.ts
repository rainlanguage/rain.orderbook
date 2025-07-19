import type { RaindexVault, RaindexVaultType } from '@rainlanguage/orderbook';

export type VaultsGroupedByType = Record<RaindexVaultType, RaindexVault[]>;
