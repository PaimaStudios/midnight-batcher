'use strict';
const __compactRuntime = require('@midnight-ntwrk/compact-runtime');
const expectedRuntimeVersionString = '0.7.0';
const expectedRuntimeVersion = expectedRuntimeVersionString.split('-')[0].split('.').map(Number);
const actualRuntimeVersion = __compactRuntime.versionString.split('-')[0].split('.').map(Number);
if (expectedRuntimeVersion[0] != actualRuntimeVersion[0]
     || (actualRuntimeVersion[0] == 0 && expectedRuntimeVersion[1] != actualRuntimeVersion[1])
     || expectedRuntimeVersion[1] > actualRuntimeVersion[1]
     || (expectedRuntimeVersion[1] == actualRuntimeVersion[1] && expectedRuntimeVersion[2] > actualRuntimeVersion[2]))
   throw new __compactRuntime.CompactError(`Version mismatch: compiled code expects ${expectedRuntimeVersionString}, runtime is ${__compactRuntime.versionString}`);
{ const MAX_FIELD = 102211695604070082112571065507755096754575920209623522239390234855480569854275933742834077002685857629445612735086326265689167708028928n;
  if (__compactRuntime.MAX_FIELD !== MAX_FIELD)
     throw new __compactRuntime.CompactError(`compiler thinks maximum field value is ${MAX_FIELD}; run time thinks it is ${__compactRuntime.MAX_FIELD}`)
}

const _descriptor_0 = new __compactRuntime.CompactTypeBytes(32);

class _ContractAddress_0 {
  alignment() {
    return _descriptor_0.alignment();
  }
  fromValue(value) {
    return {
      bytes: _descriptor_0.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_0.toValue(value.bytes);
  }
}

const _descriptor_1 = new _ContractAddress_0();

const _descriptor_2 = new __compactRuntime.CompactTypeField();

class _CurvePoint_0 {
  alignment() {
    return _descriptor_2.alignment().concat(_descriptor_2.alignment());
  }
  fromValue(value) {
    return {
      x: _descriptor_2.fromValue(value),
      y: _descriptor_2.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_2.toValue(value.x).concat(_descriptor_2.toValue(value.y));
  }
}

const _descriptor_3 = new _CurvePoint_0();

const _descriptor_4 = new __compactRuntime.CompactTypeBoolean();

class _ZswapCoinPublicKey_0 {
  alignment() {
    return _descriptor_0.alignment();
  }
  fromValue(value) {
    return {
      bytes: _descriptor_0.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_0.toValue(value.bytes);
  }
}

const _descriptor_5 = new _ZswapCoinPublicKey_0();

class _Either_0 {
  alignment() {
    return _descriptor_4.alignment().concat(_descriptor_5.alignment().concat(_descriptor_1.alignment()));
  }
  fromValue(value) {
    return {
      is_left: _descriptor_4.fromValue(value),
      left: _descriptor_5.fromValue(value),
      right: _descriptor_1.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_4.toValue(value.is_left).concat(_descriptor_5.toValue(value.left).concat(_descriptor_1.toValue(value.right)));
  }
}

const _descriptor_6 = new _Either_0();

const _descriptor_7 = new __compactRuntime.CompactTypeUnsignedInteger(340282366920938463463374607431768211455n, 16);

class _CoinInfo_0 {
  alignment() {
    return _descriptor_0.alignment().concat(_descriptor_0.alignment().concat(_descriptor_7.alignment()));
  }
  fromValue(value) {
    return {
      nonce: _descriptor_0.fromValue(value),
      color: _descriptor_0.fromValue(value),
      value: _descriptor_7.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_0.toValue(value.nonce).concat(_descriptor_0.toValue(value.color).concat(_descriptor_7.toValue(value.value)));
  }
}

const _descriptor_8 = new _CoinInfo_0();

class _ZswapCoinSecretKey_0 {
  alignment() {
    return _descriptor_0.alignment();
  }
  fromValue(value) {
    return {
      bytes: _descriptor_0.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_0.toValue(value.bytes);
  }
}

const _descriptor_9 = new _ZswapCoinSecretKey_0();

class _MerkleTreeDigest_0 {
  alignment() {
    return _descriptor_2.alignment();
  }
  fromValue(value) {
    return {
      field: _descriptor_2.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_2.toValue(value.field);
  }
}

const _descriptor_10 = new _MerkleTreeDigest_0();

class _Either_1 {
  alignment() {
    return _descriptor_4.alignment().concat(_descriptor_9.alignment().concat(_descriptor_1.alignment()));
  }
  fromValue(value) {
    return {
      is_left: _descriptor_4.fromValue(value),
      left: _descriptor_9.fromValue(value),
      right: _descriptor_1.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_4.toValue(value.is_left).concat(_descriptor_9.toValue(value.left).concat(_descriptor_1.toValue(value.right)));
  }
}

const _descriptor_11 = new _Either_1();

class _MerkleTreePathEntry_0 {
  alignment() {
    return _descriptor_10.alignment().concat(_descriptor_4.alignment());
  }
  fromValue(value) {
    return {
      sibling: _descriptor_10.fromValue(value),
      goes_left: _descriptor_4.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_10.toValue(value.sibling).concat(_descriptor_4.toValue(value.goes_left));
  }
}

const _descriptor_12 = new _MerkleTreePathEntry_0();

const _descriptor_13 = new __compactRuntime.CompactTypeVector(32, _descriptor_12);

class _MerkleTreePath_0 {
  alignment() {
    return _descriptor_0.alignment().concat(_descriptor_13.alignment());
  }
  fromValue(value) {
    return {
      leaf: _descriptor_0.fromValue(value),
      path: _descriptor_13.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_0.toValue(value.leaf).concat(_descriptor_13.toValue(value.path));
  }
}

const _descriptor_14 = new _MerkleTreePath_0();

const _descriptor_15 = new __compactRuntime.CompactTypeBytes(6);

class _PublicKeyPreimage_0 {
  alignment() {
    return _descriptor_9.alignment().concat(_descriptor_15.alignment());
  }
  fromValue(value) {
    return {
      secret_key: _descriptor_9.fromValue(value),
      sep: _descriptor_15.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_9.toValue(value.secret_key).concat(_descriptor_15.toValue(value.sep));
  }
}

const _descriptor_16 = new _PublicKeyPreimage_0();

const _descriptor_17 = new __compactRuntime.CompactTypeVector(2, _descriptor_2);

class _CoinPreimage_0 {
  alignment() {
    return _descriptor_8.alignment().concat(_descriptor_4.alignment().concat(_descriptor_0.alignment().concat(_descriptor_15.alignment())));
  }
  fromValue(value) {
    return {
      info: _descriptor_8.fromValue(value),
      data_type: _descriptor_4.fromValue(value),
      data: _descriptor_0.fromValue(value),
      sep: _descriptor_15.fromValue(value)
    }
  }
  toValue(value) {
    return _descriptor_8.toValue(value.info).concat(_descriptor_4.toValue(value.data_type).concat(_descriptor_0.toValue(value.data).concat(_descriptor_15.toValue(value.sep))));
  }
}

const _descriptor_18 = new _CoinPreimage_0();

const _descriptor_19 = new __compactRuntime.CompactTypeUnsignedInteger(18446744073709551615n, 8);

const _descriptor_20 = new __compactRuntime.CompactTypeUnsignedInteger(255n, 1);

class Contract {
  witnesses;
  constructor(...args) {
    if (args.length !== 1)
      throw new __compactRuntime.CompactError(`Contract constructor: expected 1 argument, received ${args.length}`);
    const witnesses = args[0];
    if (typeof(witnesses) !== 'object')
      throw new __compactRuntime.CompactError('first (witnesses) argument to Contract constructor is not an object');
    this.witnesses = witnesses;
    this.circuits = {
      spend: (...args_0) => {
        if (args_0.length !== 5)
          throw new __compactRuntime.CompactError(`spend: expected 5 arguments (as invoked from Typescript), received ${args_0.length}`);
        const contextOrig = args_0[0];
        const sk = args_0[1];
        const path = args_0[2];
        const coin = args_0[3];
        const rc = args_0[4];
        if (!(typeof(contextOrig) === 'object' && contextOrig.originalState != undefined && contextOrig.transactionContext != undefined))
          __compactRuntime.type_error('spend',
                                      'argument 1 (as invoked from Typescript)',
                                      'zswap.compact line 22, char 1',
                                      'CircuitContext',
                                      contextOrig)
        if (!(typeof(sk) === 'object' && typeof(sk.is_left) === 'boolean' && typeof(sk.left) === 'object' && sk.left.bytes.buffer instanceof ArrayBuffer && sk.left.bytes.BYTES_PER_ELEMENT === 1 && sk.left.bytes.length === 32 && typeof(sk.right) === 'object' && sk.right.bytes.buffer instanceof ArrayBuffer && sk.right.bytes.BYTES_PER_ELEMENT === 1 && sk.right.bytes.length === 32))
          __compactRuntime.type_error('spend',
                                      'argument 1 (argument 2 as invoked from Typescript)',
                                      'zswap.compact line 22, char 1',
                                      'struct Either[is_left: Boolean, left: struct ZswapCoinSecretKey[bytes: Bytes[32]], right: struct ContractAddress[bytes: Bytes[32]]]',
                                      sk)
        if (!(typeof(path) === 'object' && path.leaf.buffer instanceof ArrayBuffer && path.leaf.BYTES_PER_ELEMENT === 1 && path.leaf.length === 32 && Array.isArray(path.path) && path.path.length === 32 && path.path.every((t) => typeof(t) === 'object' && typeof(t.sibling) === 'object' && typeof(t.sibling.field) === 'bigint' && t.sibling.field >= 0 && t.sibling.field <= __compactRuntime.MAX_FIELD && typeof(t.goes_left) === 'boolean')))
          __compactRuntime.type_error('spend',
                                      'argument 2 (argument 3 as invoked from Typescript)',
                                      'zswap.compact line 22, char 1',
                                      'struct MerkleTreePath[leaf: Bytes[32], path: Vector[32, struct MerkleTreePathEntry[sibling: struct MerkleTreeDigest[field: Field], goes_left: Boolean]]]',
                                      path)
        if (!(typeof(coin) === 'object' && coin.nonce.buffer instanceof ArrayBuffer && coin.nonce.BYTES_PER_ELEMENT === 1 && coin.nonce.length === 32 && coin.color.buffer instanceof ArrayBuffer && coin.color.BYTES_PER_ELEMENT === 1 && coin.color.length === 32 && typeof(coin.value) === 'bigint' && coin.value >= 0 && coin.value <= 340282366920938463463374607431768211455n))
          __compactRuntime.type_error('spend',
                                      'argument 3 (argument 4 as invoked from Typescript)',
                                      'zswap.compact line 22, char 1',
                                      'struct CoinInfo[nonce: Bytes[32], color: Bytes[32], value: Unsigned Integer[<= 340282366920938463463374607431768211455]]',
                                      coin)
        if (!(typeof(rc) === 'bigint' && rc >= 0 && rc <= __compactRuntime.MAX_FIELD))
          __compactRuntime.type_error('spend',
                                      'argument 4 (argument 5 as invoked from Typescript)',
                                      'zswap.compact line 22, char 1',
                                      'Field',
                                      rc)
        const context = { ...contextOrig };
        const partialProofData = {
          input: {
            value: _descriptor_11.toValue(sk).concat(_descriptor_14.toValue(path).concat(_descriptor_8.toValue(coin).concat(_descriptor_2.toValue(rc)))),
            alignment: _descriptor_11.alignment().concat(_descriptor_14.alignment().concat(_descriptor_8.alignment().concat(_descriptor_2.alignment())))
          },
          output: undefined,
          publicTranscript: [],
          privateTranscriptOutputs: []
        };
        const result = this.#_spend_0(context,
                                      partialProofData,
                                      sk,
                                      path,
                                      coin,
                                      rc);
        partialProofData.output = { value: [], alignment: [] };
        return { result: result, context: context, proofData: partialProofData };
      },
      output: (...args_0) => {
        if (args_0.length !== 4)
          throw new __compactRuntime.CompactError(`output: expected 4 arguments (as invoked from Typescript), received ${args_0.length}`);
        const contextOrig = args_0[0];
        const pk = args_0[1];
        const coin = args_0[2];
        const rc = args_0[3];
        if (!(typeof(contextOrig) === 'object' && contextOrig.originalState != undefined && contextOrig.transactionContext != undefined))
          __compactRuntime.type_error('output',
                                      'argument 1 (as invoked from Typescript)',
                                      'zswap.compact line 63, char 1',
                                      'CircuitContext',
                                      contextOrig)
        if (!(typeof(pk) === 'object' && typeof(pk.is_left) === 'boolean' && typeof(pk.left) === 'object' && pk.left.bytes.buffer instanceof ArrayBuffer && pk.left.bytes.BYTES_PER_ELEMENT === 1 && pk.left.bytes.length === 32 && typeof(pk.right) === 'object' && pk.right.bytes.buffer instanceof ArrayBuffer && pk.right.bytes.BYTES_PER_ELEMENT === 1 && pk.right.bytes.length === 32))
          __compactRuntime.type_error('output',
                                      'argument 1 (argument 2 as invoked from Typescript)',
                                      'zswap.compact line 63, char 1',
                                      'struct Either[is_left: Boolean, left: struct ZswapCoinPublicKey[bytes: Bytes[32]], right: struct ContractAddress[bytes: Bytes[32]]]',
                                      pk)
        if (!(typeof(coin) === 'object' && coin.nonce.buffer instanceof ArrayBuffer && coin.nonce.BYTES_PER_ELEMENT === 1 && coin.nonce.length === 32 && coin.color.buffer instanceof ArrayBuffer && coin.color.BYTES_PER_ELEMENT === 1 && coin.color.length === 32 && typeof(coin.value) === 'bigint' && coin.value >= 0 && coin.value <= 340282366920938463463374607431768211455n))
          __compactRuntime.type_error('output',
                                      'argument 2 (argument 3 as invoked from Typescript)',
                                      'zswap.compact line 63, char 1',
                                      'struct CoinInfo[nonce: Bytes[32], color: Bytes[32], value: Unsigned Integer[<= 340282366920938463463374607431768211455]]',
                                      coin)
        if (!(typeof(rc) === 'bigint' && rc >= 0 && rc <= __compactRuntime.MAX_FIELD))
          __compactRuntime.type_error('output',
                                      'argument 3 (argument 4 as invoked from Typescript)',
                                      'zswap.compact line 63, char 1',
                                      'Field',
                                      rc)
        const context = { ...contextOrig };
        const partialProofData = {
          input: {
            value: _descriptor_6.toValue(pk).concat(_descriptor_8.toValue(coin).concat(_descriptor_2.toValue(rc))),
            alignment: _descriptor_6.alignment().concat(_descriptor_8.alignment().concat(_descriptor_2.alignment()))
          },
          output: undefined,
          publicTranscript: [],
          privateTranscriptOutputs: []
        };
        const result = this.#_output_0(context, partialProofData, pk, coin, rc);
        partialProofData.output = { value: [], alignment: [] };
        return { result: result, context: context, proofData: partialProofData };
      },
      sign: (...args_0) => {
        if (args_0.length !== 2)
          throw new __compactRuntime.CompactError(`sign: expected 2 arguments (as invoked from Typescript), received ${args_0.length}`);
        const contextOrig = args_0[0];
        const secret_key = args_0[1];
        if (!(typeof(contextOrig) === 'object' && contextOrig.originalState != undefined && contextOrig.transactionContext != undefined))
          __compactRuntime.type_error('sign',
                                      'argument 1 (as invoked from Typescript)',
                                      'zswap.compact line 84, char 1',
                                      'CircuitContext',
                                      contextOrig)
        if (!(typeof(secret_key) === 'object' && secret_key.bytes.buffer instanceof ArrayBuffer && secret_key.bytes.BYTES_PER_ELEMENT === 1 && secret_key.bytes.length === 32))
          __compactRuntime.type_error('sign',
                                      'argument 1 (argument 2 as invoked from Typescript)',
                                      'zswap.compact line 84, char 1',
                                      'struct ZswapCoinSecretKey[bytes: Bytes[32]]',
                                      secret_key)
        const context = { ...contextOrig };
        const partialProofData = {
          input: {
            value: _descriptor_9.toValue(secret_key),
            alignment: _descriptor_9.alignment()
          },
          output: undefined,
          publicTranscript: [],
          privateTranscriptOutputs: []
        };
        const result = this.#_sign_0(context, partialProofData, secret_key);
        partialProofData.output = { value: [], alignment: [] };
        return { result: result, context: context, proofData: partialProofData };
      }
    };
    this.impureCircuits = {
      spend: this.circuits.spend,
      output: this.circuits.output,
      sign: this.circuits.sign
    };
  }
  initialState(...args) {
    if (args.length !== 1)
      throw new __compactRuntime.CompactError(`Contract state constructor: expected 1 argument (as invoked from Typescript), received ${args.length}`);
    const constructorContext = args[0];
    if (typeof(constructorContext) !== 'object') {
      throw new __compactRuntime.CompactError(`Contract state constructor: expected 'constructorContext' in argument 1 (as invoked from Typescript) to be an object`);
    }
    if (!('initialZswapLocalState' in constructorContext)) {
      throw new __compactRuntime.CompactError(`Contract state constructor: expected 'initialZswapLocalState' in argument 1 (as invoked from Typescript)`);
    }
    if (typeof(constructorContext.initialZswapLocalState) !== 'object') {
      throw new __compactRuntime.CompactError(`Contract state constructor: expected 'initialZswapLocalState' in argument 1 (as invoked from Typescript) to be an object`);
    }
    const state = new __compactRuntime.ContractState();
    let stateValue = __compactRuntime.StateValue.newArray();
    stateValue = stateValue.arrayPush(__compactRuntime.StateValue.newNull());
    stateValue = stateValue.arrayPush(__compactRuntime.StateValue.newNull());
    stateValue = stateValue.arrayPush(__compactRuntime.StateValue.newNull());
    stateValue = stateValue.arrayPush(__compactRuntime.StateValue.newNull());
    stateValue = stateValue.arrayPush(__compactRuntime.StateValue.newNull());
    state.data = stateValue;
    state.setOperation('spend', new __compactRuntime.ContractOperation());
    state.setOperation('output', new __compactRuntime.ContractOperation());
    state.setOperation('sign', new __compactRuntime.ContractOperation());
    const context = {
      originalState: state,
      currentPrivateState: constructorContext.initialPrivateState,
      currentZswapLocalState: constructorContext.initialZswapLocalState,
      transactionContext: new __compactRuntime.QueryContext(state.data, __compactRuntime.dummyContractAddress())
    };
    const partialProofData = {
      input: { value: [], alignment: [] },
      output: undefined,
      publicTranscript: [],
      privateTranscriptOutputs: []
    };
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(0n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newArray()
                                        .arrayPush(__compactRuntime.StateValue.newBoundedMerkleTree(
                                                     new __compactRuntime.StateBoundedMerkleTree(32)
                                                   )).arrayPush(__compactRuntime.StateValue.newCell({ value: _descriptor_19.toValue(0n),
                                                                                                      alignment: _descriptor_19.alignment() })).arrayPush(__compactRuntime.StateValue.newMap(
                                                                                                                                                            new __compactRuntime.StateMap()
                                                                                                                                                          ))
                                        .encode() } },
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(2n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { dup: { n: 2 } },
                     { idx: { cached: false,
                              pushPath: false,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(0n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     'root',
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newNull().encode() } },
                     { ins: { cached: true, n: 2 } },
                     { ins: { cached: false, n: 1 } }
                    ])
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(1n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newMap(
                                        new __compactRuntime.StateMap()
                                      ).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ])
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(2n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_3.toValue({ x: 0n, y: 0n }),
                                                                            alignment: _descriptor_3.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ])
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(3n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_1.toValue({ bytes: new Uint8Array(32) }),
                                                                            alignment: _descriptor_1.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ])
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(4n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_5.toValue({ bytes: new Uint8Array(32) }),
                                                                            alignment: _descriptor_5.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ])
    state.data = context.transactionContext.state;
    return {
      currentContractState: state,
      currentPrivateState: context.currentPrivateState,
      currentZswapLocalState: context.currentZswapLocalState
    }
  }
  #_transient_hash_0(context, partialProofData, value) {
    return __compactRuntime.transientHash(_descriptor_17, value);
  }
  #_persistent_hash_0(context, partialProofData, value) {
    return __compactRuntime.persistentHash(_descriptor_18, value);
  }
  #_persistent_hash_1(context, partialProofData, value) {
    return __compactRuntime.persistentHash(_descriptor_16, value);
  }
  #_degrade_to_transient_0(context, partialProofData, x) {
    return __compactRuntime.degradeToTransient(x);
  }
  #_ec_add_0(context, partialProofData, a, b) {
    return __compactRuntime.ecAdd(a, b);
  }
  #_ec_mul_0(context, partialProofData, a, b) {
    return __compactRuntime.ecMul(a, b);
  }
  #_ec_mul_generator_0(context, partialProofData, b) {
    return __compactRuntime.ecMulGenerator(b);
  }
  #_hash_to_curve_0(context, partialProofData, value) {
    return __compactRuntime.hashToCurve(_descriptor_0, value);
  }
  #_merkle_tree_path_root_no_leaf_hash_0(context, partialProofData, path) {
    return { field:
               this.#_folder_0(context,
                               partialProofData,
                               (...args) =>
                                 this.#_merkle_tree_path_entry_root_0(...args),
                               this.#_degrade_to_transient_0(context,
                                                             partialProofData,
                                                             path.leaf),
                               path.path) };
  }
  #_merkle_tree_path_entry_root_0(context,
                                  partialProofData,
                                  recursive_digest,
                                  entry)
  {
    const left = entry.goes_left? recursive_digest : entry.sibling.field;
    const right = entry.goes_left? entry.sibling.field : recursive_digest;
    return this.#_transient_hash_0(context, partialProofData, [left, right]);
  }
  #_spend_0(context, partialProofData, sk, path, coin, rc) {
    const tmp = this.#_merkle_tree_path_root_no_leaf_hash_0(context,
                                                            partialProofData,
                                                            path);
    _descriptor_4.fromValue(Contract._query(context,
                                            partialProofData,
                                            [
                                             { dup: { n: 0 } },
                                             { idx: { cached: false,
                                                      pushPath: false,
                                                      path: [
                                                             { tag: 'value',
                                                               value: { value: _descriptor_20.toValue(0n),
                                                                        alignment: _descriptor_20.alignment() } }
                                                            ] } },
                                             { idx: { cached: false,
                                                      pushPath: false,
                                                      path: [
                                                             { tag: 'value',
                                                               value: { value: _descriptor_20.toValue(2n),
                                                                        alignment: _descriptor_20.alignment() } }
                                                            ] } },
                                             { push: { storage: false,
                                                       value: __compactRuntime.StateValue.newCell({ value: _descriptor_10.toValue(tmp),
                                                                                                    alignment: _descriptor_10.alignment() }).encode() } },
                                             'member',
                                             { popeq: { cached: true,
                                                        result: undefined } }
                                            ]).value);
    const pk = this.#_derive_public_key_0(context, partialProofData, sk.left);
    const coin_com = this.#_persistent_hash_0(context,
                                              partialProofData,
                                              { info: coin,
                                                data_type: sk.is_left,
                                                data:
                                                  sk.is_left?
                                                  pk.bytes :
                                                  sk.right.bytes,
                                                sep:
                                                  new Uint8Array([109, 100, 110, 58, 99, 99]) });
    __compactRuntime.assert(this.#_equal_0(coin_com, path.leaf),
                            'Path must be to coin commitment');
    const nul = this.#_persistent_hash_0(context,
                                         partialProofData,
                                         { info: coin,
                                           data_type: sk.is_left,
                                           data:
                                             sk.is_left?
                                             sk.left.bytes :
                                             sk.right.bytes,
                                           sep:
                                             new Uint8Array([109, 100, 110, 58, 99, 110]) });
    Contract._query(context,
                    partialProofData,
                    [
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(1n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_0.toValue(nul),
                                                                            alignment: _descriptor_0.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newNull().encode() } },
                     { ins: { cached: false, n: 1 } },
                     { ins: { cached: true, n: 1 } }
                    ]);
    if (!sk.is_left) {
      const tmp_0 = sk.right;
      Contract._query(context,
                      partialProofData,
                      [
                       { push: { storage: false,
                                 value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(3n),
                                                                              alignment: _descriptor_20.alignment() }).encode() } },
                       { push: { storage: true,
                                 value: __compactRuntime.StateValue.newCell({ value: _descriptor_1.toValue(tmp_0),
                                                                              alignment: _descriptor_1.alignment() }).encode() } },
                       { ins: { cached: false, n: 1 } }
                      ]);
    }
    const color_base = this.#_hash_to_curve_0(context,
                                              partialProofData,
                                              coin.color);
    const pedersen_blinding = this.#_ec_mul_generator_0(context,
                                                        partialProofData,
                                                        rc);
    const pedersen_commit = this.#_ec_mul_0(context,
                                            partialProofData,
                                            color_base,
                                            coin.value);
    const tmp_1 = this.#_ec_add_0(context,
                                  partialProofData,
                                  pedersen_blinding,
                                  pedersen_commit);
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(2n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_3.toValue(tmp_1),
                                                                            alignment: _descriptor_3.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ]);
  }
  #_derive_public_key_0(context, partialProofData, secret_key) {
    return { bytes:
               this.#_persistent_hash_1(context,
                                        partialProofData,
                                        { secret_key: secret_key,
                                          sep:
                                            new Uint8Array([109, 100, 110, 58, 112, 107]) }) };
  }
  #_output_0(context, partialProofData, pk, coin, rc) {
    const coin_com = this.#_persistent_hash_0(context,
                                              partialProofData,
                                              { info: coin,
                                                data_type: pk.is_left,
                                                data:
                                                  pk.is_left?
                                                  pk.left.bytes :
                                                  pk.right.bytes,
                                                sep:
                                                  new Uint8Array([109, 100, 110, 58, 99, 99]) });
    Contract._query(context,
                    partialProofData,
                    [
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(0n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(0n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { dup: { n: 2 } },
                     { idx: { cached: false,
                              pushPath: false,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(1n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_0.toValue(coin_com),
                                                                            alignment: _descriptor_0.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } },
                     { ins: { cached: true, n: 1 } },
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(1n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { addi: { immediate: 1 } },
                     { ins: { cached: true, n: 1 } },
                     { idx: { cached: false,
                              pushPath: true,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(2n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     { dup: { n: 2 } },
                     { idx: { cached: false,
                              pushPath: false,
                              path: [
                                     { tag: 'value',
                                       value: { value: _descriptor_20.toValue(0n),
                                                alignment: _descriptor_20.alignment() } }
                                    ] } },
                     'root',
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newNull().encode() } },
                     { ins: { cached: false, n: 1 } },
                     { ins: { cached: true, n: 2 } }
                    ]);
    if (!pk.is_left) {
      const tmp = pk.right;
      Contract._query(context,
                      partialProofData,
                      [
                       { push: { storage: false,
                                 value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(3n),
                                                                              alignment: _descriptor_20.alignment() }).encode() } },
                       { push: { storage: true,
                                 value: __compactRuntime.StateValue.newCell({ value: _descriptor_1.toValue(tmp),
                                                                              alignment: _descriptor_1.alignment() }).encode() } },
                       { ins: { cached: false, n: 1 } }
                      ]);
    }
    const color_base = this.#_hash_to_curve_0(context,
                                              partialProofData,
                                              coin.color);
    const pedersen_blinding = this.#_ec_mul_generator_0(context,
                                                        partialProofData,
                                                        rc);
    const pedersen_commit = this.#_ec_mul_0(context,
                                            partialProofData,
                                            color_base,
                                            coin.value);
    const tmp_0 = this.#_ec_add_0(context,
                                  partialProofData,
                                  pedersen_blinding,
                                  pedersen_commit);
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(2n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_3.toValue(tmp_0),
                                                                            alignment: _descriptor_3.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ]);
  }
  #_sign_0(context, partialProofData, secret_key) {
    const tmp = this.#_derive_public_key_0(context, partialProofData, secret_key);
    Contract._query(context,
                    partialProofData,
                    [
                     { push: { storage: false,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_20.toValue(4n),
                                                                            alignment: _descriptor_20.alignment() }).encode() } },
                     { push: { storage: true,
                               value: __compactRuntime.StateValue.newCell({ value: _descriptor_5.toValue(tmp),
                                                                            alignment: _descriptor_5.alignment() }).encode() } },
                     { ins: { cached: false, n: 1 } }
                    ]);
  }
  #_folder_0(context, partialProofData, f, x, a0)
  {
    for (let i = 0; i < 32; i++) x = f(context, partialProofData, x, a0[i]);
    return x;
  }
  #_equal_0(x0, y0) {
    if (!x0.every((x, i) => y0[i] === x)) return false;
    return true;
  }
  static _query(context, partialProofData, prog) {
    var res;
    try {
      res = context.transactionContext.query(prog, __compactRuntime.CostModel.dummyCostModel());
    } catch (err) {
      throw new __compactRuntime.CompactError(err.toString());
    }
    context.transactionContext = res.context;
    var reads = res.events.filter((e) => e.tag === 'read');
    var i = 0;
    partialProofData.publicTranscript = partialProofData.publicTranscript.concat(prog.map((op) => {
      if(typeof(op) === 'object' && 'popeq' in op) {
        return { popeq: {
          ...op.popeq,
          result: reads[i++].content,
        } };
      } else {
        return op;
      }
    }));
    if(res.events.length == 1 && res.events[0].tag === 'read') {
      return res.events[0].content;
    } else {
      return res.events;
    }
  }
}
function ledger(state) {
  const context = {
    originalState: state,
    transactionContext: new __compactRuntime.QueryContext(state, __compactRuntime.dummyContractAddress())
  };
  const partialProofData = {
    input: { value: [], alignment: [] },
    output: undefined,
    publicTranscript: [],
    privateTranscriptOutputs: []
  };
  return {
  };
}
const _emptyContext = {
  originalState: new __compactRuntime.ContractState(),
  transactionContext: new __compactRuntime.QueryContext(new __compactRuntime.ContractState().data, __compactRuntime.dummyContractAddress())
};
const _dummyContract = new Contract({ });
const pureCircuits = { };
const contractReferenceLocations = { tag: 'publicLedgerArray', indices: { } };
exports.Contract = Contract;
exports.ledger = ledger;
exports.pureCircuits = pureCircuits;
exports.contractReferenceLocations = contractReferenceLocations;
//# sourceMappingURL=index.cjs.map
