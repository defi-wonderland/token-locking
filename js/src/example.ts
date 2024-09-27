import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import fs from 'fs';
import { Numberu64, generateRandomSeed } from './utils';
import { Schedule } from './state';
import { create, TOKEN_VESTING_PROGRAM_ID } from './main';
import { signAndSendInstructions } from '@bonfida/utils';

/**
 *
 * Simple example of a linear unlock.
 *
 * This is just an example, please be careful using the vesting contract and test it first with test tokens.
 *
 */

/** Path to your wallet */
const WALLET_PATH = '';
const wallet = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync(WALLET_PATH).toString())),
);

/** There are better way to generate an array of dates but be careful as it's irreversible */
const DATE = new Date(2022, 12);

/** Info about the desintation */
const DESTINATION_OWNER = new PublicKey('');
const DESTINATION_TOKEN_ACCOUNT = new PublicKey('');

/** Token info */
const MINT = new PublicKey('');
const DECIMALS = 0;

/** Info about the source */
const SOURCE_TOKEN_ACCOUNT = new PublicKey('');

/** Amount to give per schedule */
const LOCKED_AMOUNT = 0;

/** Your RPC connection */
const connection = new Connection('');

/** Do some checks before sending the tokens */
const checks = async () => {
  const tokenInfo = await connection.getParsedAccountInfo(
    DESTINATION_TOKEN_ACCOUNT,
  );

  // @ts-ignore
  const parsed = tokenInfo.value.data.parsed;
  if (parsed.info.mint !== MINT.toBase58()) {
    throw new Error('Invalid mint');
  }
  if (parsed.info.owner !== DESTINATION_OWNER.toBase58()) {
    throw new Error('Invalid owner');
  }
  if (parsed.info.tokenAmount.decimals !== DECIMALS) {
    throw new Error('Invalid decimals');
  }
};

/** Function that locks the tokens */
const lock = async () => {
  await checks();
  const schedule: Schedule = new Schedule(
    /** Has to be in seconds */
    // @ts-ignore
    new Numberu64(60),
    /** Don't forget to add decimals */
    // @ts-ignore
    new Numberu64(LOCKED_AMOUNT * Math.pow(10, DECIMALS)),
  );
  const seed = generateRandomSeed();

  console.log(`Seed: ${seed}`);

  const instruction = await create(
    connection,
    TOKEN_VESTING_PROGRAM_ID,
    Buffer.from(seed),
    wallet.publicKey,
    wallet.publicKey,
    SOURCE_TOKEN_ACCOUNT,
    DESTINATION_TOKEN_ACCOUNT,
    MINT,
    schedule,
  );

  const tx = await signAndSendInstructions(connection, [], wallet, instruction);

  console.log(`Transaction: ${tx}`);

  const txInfo = await connection.getConfirmedTransaction(tx, 'confirmed');
  if (txInfo && !txInfo.meta?.err) {
    console.log(
      txInfo?.transaction.instructions[2].data.slice(1, 32 + 1).toString('hex'),
    );
  } else {
    throw new Error('Transaction not confirmed.');
  }
};

lock();
