import { Address, BigInt, Bytes, log } from "@graphprotocol/graph-ts";
import { AddOrderOrderStruct } from "../generated/OrderBook/OrderBook";
import { getEvenHex } from "./utils";

class JsonString {
  _obj: Map<string, string>;

  constructor(map_: Map<string, string>) {
    this._obj = map_;
  }

  stringify(): string {
    const keys = this._obj.keys();
    const objs: string[] = new Array<string>(keys.length);

    for (let i: i32 = 0; i < keys.length; i++) {
      const key = keys[i];
      const value = this._obj.get(key);
      // "Array"
      if (value.startsWith("[") && value.endsWith("]")) {
        //
        objs[i] = `"${key}":${value}`;
      } else {
        objs[i] = `"${key}":"${value}"`;
      }
    }

    return `{${objs.join(",")}}`;
  }
}

/**
 * Generate a JSON string for a given order to be ready to use with tools
 */

export class OrderString extends JsonString {
  constructor(order_: AddOrderOrderStruct) {
    const _map: Map<string, string> = new Map();

    const evaluable_ = new Evaluable_String(
      order_.evaluable.interpreter,
      order_.evaluable.store,
      order_.evaluable.expression
    );

    const validInputsArr: string[] = [];
    const validOutputsArr: string[] = [];

    const validInputs_ = order_.validInputs;
    const validOutputs_ = order_.validOutputs;

    for (let i = 0; i < validInputs_.length; i++) {
      const input_ = validInputs_[i];
      const io_ = new IO_String(input_.token, input_.decimals, input_.vaultId);

      validInputsArr.push(io_.stringify());
    }

    for (let i = 0; i < validOutputs_.length; i++) {
      const output_ = validOutputs_[i];
      const io_ = new IO_String(
        output_.token,
        output_.decimals,
        output_.vaultId
      );

      validOutputsArr.push(io_.stringify());
    }

    _map.set("owner", getEvenHex(order_.owner.toHex()));
    _map.set("handleIO", (order_.handleIO as bool).toString());
    _map.set("evaluable", evaluable_.stringify());
    _map.set("validInputs", `[${validInputsArr.join(",")}]`);
    _map.set("validOutputs", `[${validOutputsArr.join(",")}]`);

    super(_map);
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

class IO_String extends JsonString {
  constructor(token_: Address, decimals_: number, vaultId_: BigInt) {
    const _map: Map<string, string> = new Map();

    _map.set("token", getEvenHex(token_.toHex()));
    _map.set("decimals", decimals_.toString().split(".")[0]);
    _map.set("vaultId", getEvenHex(vaultId_.toHex()));

    super(_map);
  }
}

class Evaluable_String extends JsonString {
  constructor(interpreter_: Address, store_: Address, expression_: Address) {
    const _map: Map<string, string> = new Map();

    _map.set("interpreter", getEvenHex(interpreter_.toHex()));
    _map.set("store", getEvenHex(store_.toHex()));
    _map.set("expression", getEvenHex(expression_.toHex()));

    super(_map);
  }
}

export class ExpressionJSONString extends JsonString {
  constructor(sources_: Bytes[], constants_: BigInt[]) {
    const _map: Map<string, string> = new Map();

    const sources_string = sources_.map<string>(
      (x: Bytes): string => `"${x.toHexString()}"`
    );
    const constants_string = constants_.map<string>(
      (x): string => `"${x.toHexString()}"`
    );

    _map.set("sources", `[${sources_string.join(",")}]`);
    _map.set("constants", `[${constants_string.join(",")}]`);

    super(_map);
  }
}
