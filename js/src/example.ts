import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import fs from 'fs';
import { Numberu64, generateRandomSeed } from './utils';
import { CreateSchedule } from './state';
import {
  create,
  VESTING_PROGRAM_ID,
  DEVNET_VESTING_PROGRAM_ID,
  TOKEN_MINT,
  initializeUnlock,
  unlock,
} from './main';
import { signAndSendInstructions } from '@bonfida/utils';

/**
 *
 * This is just an example, please be careful using the vesting contract and test it first with test tokens.
 *
 */

/** Path to your wallet */
const WALLET_PATH = '';
const wallet = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync(WALLET_PATH).toString())),
);

/** Info about the desintation */
const LOCK_OWNER = new PublicKey('');
const LOCK_OWNER_TOKEN_ACCOUNT = new PublicKey('');

/** Info about the deposit (to interact with) */
const LOCK_SEED = '';

/** Token info */
const DECIMALS = 9;

/** Amount to lock */
const LOCKED_AMOUNT = 10;

/** Your RPC connection */
const connection = new Connection('');
const DEVNET = true;
const program = DEVNET ? DEVNET_VESTING_PROGRAM_ID : VESTING_PROGRAM_ID;

/** Do some checks before sending the tokens */
const checks = async () => {
  const tokenInfo = await connection.getParsedAccountInfo(
    LOCK_OWNER_TOKEN_ACCOUNT,
  );

  // @ts-ignore
  const parsed = tokenInfo.value.data.parsed;
  if (parsed.info.mint !== TOKEN_MINT.toBase58()) {
    throw new Error('Invalid mint');
  }
  if (parsed.info.owner !== LOCK_OWNER.toBase58()) {
    throw new Error('Invalid owner');
  }
  if (parsed.info.tokenAmount.decimals !== DECIMALS) {
    throw new Error('Invalid decimals');
  }
};

/** Function that locks the tokens */
const lock = async () => {
  await checks();
  const schedule: CreateSchedule = new CreateSchedule(
    /** Has to be 0 | 3 | 6 | 12 mths (in seconds) */
    // @ts-ignore
    new Numberu64(0), // unlocked with withdrawal period
    // @ts-ignore
    new Numberu64(LOCKED_AMOUNT * Math.pow(10, DECIMALS)),
  );
  const seed = generateRandomSeed();

  console.log(`Seed: ${seed}`);

  const instruction = await create(
    connection,
    program,
    // @ts-ignore
    Buffer.from(seed),
    wallet.publicKey,
    LOCK_OWNER_TOKEN_ACCOUNT,
    schedule,
  );

  const tx = await signAndSendInstructions(connection, [], wallet, instruction);

  console.log(`Transaction: ${tx}`);
};

const initUnlock = async () => {
  await checks();

  const instruction = await initializeUnlock(
    connection,
    VESTING_PROGRAM_ID,
    // @ts-ignore
    Buffer.from(LOCK_SEED),
  );

  const tx = await signAndSendInstructions(connection, [], wallet, instruction);

  console.log(`Transaction: ${tx}`);
};

const withdraw = async () => {
  await checks();

  const instruction = await unlock(
    connection,
    program,
    // @ts-ignore
    Buffer.from(LOCK_SEED),
  );

  const tx = await signAndSendInstructions(connection, [], wallet, instruction);

  console.log(`Transaction: ${tx}`);
};

lock();
// initUnlock();
// withdraw();
