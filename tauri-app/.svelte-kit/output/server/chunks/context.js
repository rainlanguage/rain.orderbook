import { s as setContext, g as getContext } from "./ssr.js";
const ACCOUNT_KEY = "account_key";
const getAccountContext = () => {
  const account = getContext(ACCOUNT_KEY);
  if (!account) {
    throw new Error("No account was found in Svelte context. Did you forget to wrap your component with WalletProvider?");
  }
  return account;
};
const setAccountContext = (account) => {
  setContext(ACCOUNT_KEY, account);
};
export {
  getAccountContext as g,
  setAccountContext as s
};
//# sourceMappingURL=context.js.map
