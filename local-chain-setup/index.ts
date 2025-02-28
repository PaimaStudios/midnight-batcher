import type { Wallet } from "@midnight-ntwrk/wallet-api";
import { WalletBuilder } from "@midnight-ntwrk/wallet";
import { nativeToken, NetworkId } from "@midnight-ntwrk/zswap";
import Rx from "rxjs";
import { exit } from "node:process";
import { spawn, exec, ChildProcessWithoutNullStreams, ChildProcess, execSync } from "node:child_process";
import { promisify } from 'util';

const children: ChildProcess[] = [];
process.on('SIGINT', () => {
  for (const child of children) {
    child.kill('SIGINT')
  }
})

const execP = promisify(exec);

// const { stdout, stderr } = await execP('docker volume ls');

// const COMPOSE_PROJECT_NAME = process.env.COMPOSE_PROJECT_NAME;

// const volumeExists = stdout.split("\n").map(s => s.split(" ").filter(s => s.length)).some(s => s[1] === `${COMPOSE_PROJECT_NAME}_midnight-data-undeployed`);
console.log("volume not found, starting setup of new network");


const GENESIS_MINT_WALLET_SEED =
  "0000000000000000000000000000000000000000000000000000000000000001";

const wallet = await WalletBuilder.buildFromSeed(
  "http://127.0.0.1:8088/api/v1/graphql",
  "ws://127.0.0.1:8088/api/v1/graphql/ws",
  "http://127.0.0.1:6300",
  "http://127.0.0.1:9944",
  GENESIS_MINT_WALLET_SEED,
  NetworkId.Undeployed
);

const waitForFunds = (wallet: Wallet) =>
  Rx.firstValueFrom(
    wallet.state().pipe(
      Rx.throttleTime(10_000),
      Rx.tap((state) => {
        const scanned = state.syncProgress?.synced ?? 0n;
        const total = state.syncProgress?.total.toString() ?? "unknown number";

        console.log(`Scanned ${scanned}, total: ${total}`);
      }),
      Rx.filter((state) => {
        // Let's allow progress only if wallet is close enough
        const synced = state.syncProgress?.synced ?? 0n;
        const total = state.syncProgress?.total ?? 1_000n;
        return total - synced < 100n;
      }),
      Rx.map((s) => s.balances[nativeToken()] ?? 0n),
      Rx.filter((balance) => balance > 0n)
    )
  );

wallet.start();

const state = await Rx.firstValueFrom(wallet.state());
let balance = state.balances[nativeToken()];
if (balance === undefined || balance === 0n) {
  console.log("Waiting for wallet to sync up");
  balance = await waitForFunds(wallet);
  console.log("balance", balance);
}

console.log("Sending funds");

const utxos = 4;

const receiverAddresses = Array.from({ length: utxos }).map(_ =>
  "25390c97cda75b7db1b24aa1e34910234b58ca0f1d66f847438d5d97d40f7760|0300d491742496c85185533d20d9eb4cabfe94e2f53670abea6ec145d0b7c728e28b49eac08af8691451dc7d1380dff0b0cc20559b112098610b")
  ;

let i = 0;

for (const receiverAddress of receiverAddresses) {
  console.log(`Sending utxo ${i} of ${utxos} to batcher address`);
  const transferRecipe = await wallet.transferTransaction([
    {
      amount: 10000000000n,
      receiverAddress: receiverAddress,
      type: nativeToken(),
    },
  ]);

  const transaction = await wallet.proveTransaction(transferRecipe);
  console.log("Proved transaction");

  await wallet.submitTransaction(transaction);
  console.log("Submitted transaction");

  i++;
}

wallet.close();
