import { Address, BigInt, Bytes, log } from "@graphprotocol/graph-ts";
import { getEvenHexString, JsonString } from "@rainprotocol/subgraph-utils";
import { AddOrderOrderStruct } from "../../../generated/OrderBook/OrderBook";

/**
 * Generate a JSON string for a given order to be ready to use with tools
 */

export class OrderString extends JsonString {
  constructor(order: AddOrderOrderStruct) {
    const map: Map<string, string> = new Map();

    const evaluable = new EvaluableString(
      order.evaluable.interpreter,
      order.evaluable.store,
      order.evaluable.expression
    );

    const validInputsArr: string[] = [];
    const validOutputsArr: string[] = [];

    const validInputsOrder = order.validInputs;
    const validOutputsOrder = order.validOutputs;

    for (let i = 0; i < validInputsOrder.length; i++) {
      const input = validInputsOrder[i];
      const io = new IOString(input.token, input.decimals, input.vaultId);

      validInputsArr.push(io.stringify());
    }

    for (let i = 0; i < validOutputsOrder.length; i++) {
      const output = validOutputsOrder[i];
      const io = new IOString(
        output.token,
        output.decimals,
        output.vaultId
      );

      validOutputsArr.push(io.stringify());
    }

    map.set("owner", getEvenHexString(order.owner.toHex()));
    map.set("handleIo", (order.handleIO as bool).toString());
    map.set("evaluable", evaluable.stringify());
    map.set("validInputs", `[${validInputsArr.join(",")}]`);
    map.set("validOutputs", `[${validOutputsArr.join(",")}]`);

    super(map);
  }

  stringify(): string {
    const keys = this._obj.keys();
    const objs: string[] = new Array<string>(keys.length);

    for (let i: i32 = 0; i < keys.length; i++) {
      const key = keys[i];
      const value = this._obj.get(key);
      if (key == "owner") {
        objs[i] = `"${key}":"${value}"`;
      } else {
        objs[i] = `"${key}":${value}`;
      }
    }

    return `{${objs.join(",")}}`;
  }
}

class IOString extends JsonString {
  constructor(token: Address, decimals: number, vaultId: BigInt) {
    const map: Map<string, string> = new Map();

    map.set("token", getEvenHexString(token.toHex()));
    map.set("decimals", decimals.toString().split(".")[0]);
    map.set("vaultId", vaultId.toHex());

    super(map);
  }
}

class EvaluableString extends JsonString {
  constructor(interpreter: Address, store: Address, expression: Address) {
    const map: Map<string, string> = new Map();

    map.set("interpreter", getEvenHexString(interpreter.toHex()));
    map.set("store", getEvenHexString(store.toHex()));
    map.set("expression", getEvenHexString(expression.toHex()));

    super(map);
  }
}

export class ExpressionJSONString extends JsonString {
  constructor(bytecode: Bytes, constants: BigInt[], minOutputs: BigInt[]) {
    const map: Map<string, string> = new Map();

    const minOutputs_string = minOutputs.map<string>(
      (x): string => `"${x.toHexString()}"`
    );
    const constants_string = constants.map<string>(
      (x): string => `"${x.toHexString()}"`
    );

    map.set("bytecode", bytecode.toHexString());
    map.set("constants", `[${constants_string.join(",")}]`);
    map.set("minOutputs", `[${minOutputs_string.join(",")}]`);

    super(map);
  }
}
