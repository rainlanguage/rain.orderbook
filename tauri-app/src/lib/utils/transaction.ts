import * as chains from 'viem/chains';

export const formatBlockExplorerTransactionUrl = (chainId: number, hash: string) => {
  const chain = Object.values(chains).find((chain) => chain.id === chainId);
  if (chain?.blockExplorers) {
    return chain.blockExplorers.default.url + `/tx/${hash}`;
  } else {
    return '';
  }
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function formatEthersTransactionError(e: any) {
  if (typeof e === 'object') {
    return `Transaction failed, error: 
    ${JSON.stringify(e)}`;
  } else if (typeof e === 'string') return e;
  else if (e instanceof Error) return e.message;
  else {
    return 'Transaction failed!';
  }
}
