import { type TokenInfo } from "@rainlanguage/orderbook/js_api";

export interface ExtendedTokenInfo extends TokenInfo {
  logoUri?: string;
  chainId?: number;
}



