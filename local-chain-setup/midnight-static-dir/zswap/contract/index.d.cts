import type * as __compactRuntime from '@midnight-ntwrk/compact-runtime';

export type Witnesses<T> = {
}

export type ImpureCircuits<T> = {
  spend(context: __compactRuntime.CircuitContext<T>,
        sk: { is_left: boolean,
              left: { bytes: Uint8Array },
              right: { bytes: Uint8Array }
            },
        path: { leaf: Uint8Array,
                path: { sibling: { field: bigint }, goes_left: boolean }[]
              },
        coin: { nonce: Uint8Array, color: Uint8Array, value: bigint },
        rc: bigint): __compactRuntime.CircuitResults<T, void>;
  output(context: __compactRuntime.CircuitContext<T>,
         pk: { is_left: boolean,
               left: { bytes: Uint8Array },
               right: { bytes: Uint8Array }
             },
         coin: { nonce: Uint8Array, color: Uint8Array, value: bigint },
         rc: bigint): __compactRuntime.CircuitResults<T, void>;
  sign(context: __compactRuntime.CircuitContext<T>,
       secret_key: { bytes: Uint8Array }): __compactRuntime.CircuitResults<T, void>;
}

export type PureCircuits = {
}

export type Circuits<T> = {
  spend(context: __compactRuntime.CircuitContext<T>,
        sk: { is_left: boolean,
              left: { bytes: Uint8Array },
              right: { bytes: Uint8Array }
            },
        path: { leaf: Uint8Array,
                path: { sibling: { field: bigint }, goes_left: boolean }[]
              },
        coin: { nonce: Uint8Array, color: Uint8Array, value: bigint },
        rc: bigint): __compactRuntime.CircuitResults<T, void>;
  output(context: __compactRuntime.CircuitContext<T>,
         pk: { is_left: boolean,
               left: { bytes: Uint8Array },
               right: { bytes: Uint8Array }
             },
         coin: { nonce: Uint8Array, color: Uint8Array, value: bigint },
         rc: bigint): __compactRuntime.CircuitResults<T, void>;
  sign(context: __compactRuntime.CircuitContext<T>,
       secret_key: { bytes: Uint8Array }): __compactRuntime.CircuitResults<T, void>;
}

export type Ledger = {
}

export type ContractReferenceLocations = any;

export declare const contractReferenceLocations : ContractReferenceLocations;

export declare class Contract<T, W extends Witnesses<T> = Witnesses<T>> {
  witnesses: W;
  circuits: Circuits<T>;
  impureCircuits: ImpureCircuits<T>;
  constructor(witnesses: W);
  initialState(context: __compactRuntime.ConstructorContext<T>): __compactRuntime.ConstructorResult<T>;
}

export declare function ledger(state: __compactRuntime.StateValue): Ledger;
export declare const pureCircuits: PureCircuits;
