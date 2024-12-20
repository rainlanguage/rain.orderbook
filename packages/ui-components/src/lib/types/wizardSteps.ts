import type { DotrainOrderGui, GuiDeposit, GuiFieldDefinition, SelectTokens, TokenInfos, Vault } from '@rainlanguage/orderbook/js_api';

export type WizardStep =
    | { type: 'tokens'; data: { token: string; gui: DotrainOrderGui; selectTokens: SelectTokens } }
    | { type: 'fields'; data: { fieldDefinition: GuiFieldDefinition; gui: DotrainOrderGui } }
    | { type: 'deposits'; data: { deposit: GuiDeposit; gui: DotrainOrderGui; tokenInfos: TokenInfos } }
    | { type: 'tokenInput'; data: { input: Vault; gui: DotrainOrderGui; tokenInfos: TokenInfos; i: number; inputVaultIds: string[] } }
    | { type: 'tokenOutput'; data: { output: Vault; gui: DotrainOrderGui; tokenInfos: TokenInfos; i: number; outputVaultIds: string[] } };