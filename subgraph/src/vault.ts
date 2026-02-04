import { Address, Bytes, crypto, dataSource } from "@graphprotocol/graph-ts";
import { Vault, VaultList } from "../generated/schema";
import { getERC20Entity } from "./erc20";
import { Float, getCalculator } from "./float";
import { ethereum } from "@graphprotocol/graph-ts"
import { Multicall3 } from "../generated/OrderBook/Multicall3";

export const VAULT_LIST_ID = "SINGLETON";
export const MUTLICALL3_ADDRESS = "0xcA11bde05977b3631167028862bE2a173976CA11";
export const ZERO_BYTES_32 = "0x0000000000000000000000000000000000000000000000000000000000000000";

export type VaultId = Bytes;

export function vaultEntityId(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: VaultId,
  token: Bytes
): Bytes {
  let bytes = orderbook.concat(owner.concat(token.concat(vaultId)));
  return Bytes.fromByteArray(crypto.keccak256(bytes));
}

export function createEmptyVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: VaultId,
  token: Bytes
): Vault {
  getVaultList(); // make sure vault list exists
  let vault = new Vault(vaultEntityId(orderbook, owner, vaultId, token));
  vault.orderbook = orderbook;
  vault.vaultId = vaultId;
  vault.token = getERC20Entity(token);
  vault.owner = owner;
  vault.balance = Bytes.fromHexString(ZERO_BYTES_32);
  vault.vaultList = VAULT_LIST_ID;
  vault.save();
  return vault;
}

export function getVault(
  orderbook: Bytes,
  owner: Bytes,
  vaultId: Bytes,
  token: Bytes
): Vault {
  let vault = Vault.load(vaultEntityId(orderbook, owner, vaultId, token));
  if (vault == null) {
    vault = createEmptyVault(orderbook, owner, vaultId, token);
  }
  return vault;
}

export class VaultBalanceChange {
  oldVaultBalance: Float;
  newVaultBalance: Float;
}

export function handleVaultBalanceChange(
  orderbook: Bytes,
  vaultId: Bytes,
  token: Bytes,
  amount: Float,
  owner: Bytes
): VaultBalanceChange {
  const calculator = getCalculator();

  let vault = getVault(orderbook, owner, vaultId, token);
  let oldVaultBalance = vault.balance;
  vault.balance = calculator.add(oldVaultBalance, amount);
  vault.save();

  return {
    oldVaultBalance,
    newVaultBalance: vault.balance,
  };
}

export function getVaultList(): VaultList {
  let vaultList = VaultList.load(VAULT_LIST_ID);
  if (!vaultList) {
    vaultList = new VaultList(VAULT_LIST_ID);
    vaultList.save();
  }
  return vaultList;
}

// updates vaultless for all vautless vaults using multicall
// this is used in block handler and is updated at each block
export function handleVaultlessBalance(): void {  
  // Get the OrderBook  and multicall3 contract instance
  const orderBookAddress = dataSource.address()
  const multicall3 = Multicall3.bind(Address.fromString(MUTLICALL3_ADDRESS));

  // Load all vaults from the store
  const BATCH_SIZE = 1000;
  const vaultList = getVaultList().vaults.load();
  const vaultlessVaultsBatch: Vault[][] = [[]];

  for (let i = 0; i < vaultList.length; i++) {
    const vault = vaultList[i];

    // skip non vautless vaults
    if (vault.vaultId.notEqual(Bytes.fromHexString(ZERO_BYTES_32))) continue;

    if (vaultlessVaultsBatch[vaultlessVaultsBatch.length - 1].length < BATCH_SIZE) {
      vaultlessVaultsBatch[vaultlessVaultsBatch.length - 1].push(vault)
    } else {
      vaultlessVaultsBatch.push([vault]);
    }
  }

  // Batch calls using tryAggregate for better performance
  const batchCalls: ethereum.Tuple[][] = [];
  for (let i = 0; i < vaultlessVaultsBatch.length; i++) {
    const vaultlessVaults = vaultlessVaultsBatch[i];
    const calls: ethereum.Tuple[] = [];
    for (let j = 0; j < vaultlessVaults.length; j++) {
      let vault = vaultlessVaults[j];
      
      // Encode vaultBalance2(address owner, address token, bytes32 vaultId) call
      const callData = ethereum.encode(
        ethereum.Value.fromTuple(
          changetype<ethereum.Tuple>([
            ethereum.Value.fromAddress(Address.fromBytes(vault.owner)),
            ethereum.Value.fromAddress(Address.fromBytes(vault.token)),
            ethereum.Value.fromFixedBytes(vault.vaultId)
          ])
        )
      )!;
      const call3 = changetype<ethereum.Tuple>([
        ethereum.Value.fromAddress(orderBookAddress),
        ethereum.Value.fromBoolean(true), // allowFailure = true
        ethereum.Value.fromBytes(callData)
      ]);
      calls.push(call3);
    }
    batchCalls.push(calls);
  }

  for (let i = 0; i < batchCalls.length; i++) {
    // Make multicall
    const result = multicall3.tryCall(
      "aggregate3",
      "aggregate3((address,bool,bytes)[]):((bool,bytes)[])",
      [ethereum.Value.fromTupleArray(batchCalls[i])]
    );
    
    if (result.reverted) continue;
    const results = result.value[0].toTupleArray<ethereum.Tuple>();
    if (results.length !== vaultlessVaultsBatch[i].length) continue;

    for (let j = 0; j < vaultlessVaultsBatch[i].length; j++) {
      const success = results[j][0].toBoolean();
      const returnData = results[j][1].toBytes();
      if (!success) continue;
      const decoded = ethereum.decode('bytes32', returnData);
      if (decoded) {
        vaultlessVaultsBatch[i][j].balance = decoded.toBytes();
        vaultlessVaultsBatch[i][j].save();
      }
    }
  }
}
