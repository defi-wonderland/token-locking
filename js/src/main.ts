import {
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  TransactionInstruction,
  Connection,
} from '@solana/web3.js';
import {
  createAssociatedTokenAccountInstruction,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import {
  createCreateInstruction,
  createInitInstruction,
  createUnlockInstruction,
  createInitializeUnlockInstruction,
} from './instructions';
import { ContractInfo, CreateSchedule } from './state';
import bs58 from 'bs58';

/**
 * The vesting schedule program ID
 */
export const VESTING_PROGRAM_ID = new PublicKey(
  'HE6bCtjsrra8DRbJnexKoVPSr5dYs57s3cuGHfotiQbq'
);

export const TOKEN_MINT = new PublicKey(
  '5k84VjAKoGPXa7ias1BNgKUrX7e61eMPWhZDqsiD4Bpe'
);

export const DEVNET_VESTING_PROGRAM_ID = new PublicKey(
  'B6o6erKW2Vi9Nidtv4wfT8JRtdFS2W5GX1V9bJEVr9Lv'
);

export const DEVNET_TOKEN_MINT = new PublicKey(
  'FrnSwyMzw2u6DB2bQUTpia9mRHqeujdUF2bomY8Zt5BX',
);

/**
 * This function can be used to lock tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param payer The fee payer of the transaction
 * @param possibleSourceTokenPubkey The source token account (i.e where locked tokens are originating from), if null it defaults to the ATA
 * @param schedule The vesting schedule
 * @returns An array of `TransactionInstruction`
 */
export async function create(
  connection: Connection,
  programId: PublicKey,
  seedWord: Buffer | Uint8Array,
  payer: PublicKey,
  possibleSourceTokenPubkey: PublicKey | null,
  schedule: CreateSchedule,
): Promise<Array<TransactionInstruction>> {
  // If no source token account was given, use the associated source account
  if (possibleSourceTokenPubkey == null) {
    possibleSourceTokenPubkey = await getAssociatedTokenAddress(
      isDevnetConnection(connection)? DEVNET_TOKEN_MINT : TOKEN_MINT,
      payer,
      true,
    );
  }

  // Find the non reversible public key for the vesting contract via the seed
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );

  const vestingTokenAccountKey = await getAssociatedTokenAddress(
    isDevnetConnection(connection)? DEVNET_TOKEN_MINT : TOKEN_MINT,
    vestingAccountKey,
    true,
  );

  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  console.log(
    'Vesting contract account pubkey: ',
    vestingAccountKey.toBase58(),
  );

  console.log('contract ID: ', bs58.encode(seedWord));

  const check_existing = await connection.getAccountInfo(vestingAccountKey);
  if (!!check_existing) {
    throw 'Contract already exists.';
  }

  let instruction = [
    createInitInstruction(
      SystemProgram.programId,
      programId,
      payer,
      vestingAccountKey,
      [seedWord],
    ),
    createAssociatedTokenAccountInstruction(
      payer,
      vestingTokenAccountKey,
      vestingAccountKey,
      isDevnetConnection(connection)? DEVNET_TOKEN_MINT : TOKEN_MINT,
    ),
    createCreateInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      SYSVAR_CLOCK_PUBKEY,
      vestingAccountKey,
      vestingTokenAccountKey,
      payer,
      possibleSourceTokenPubkey,
      schedule,
      [seedWord],
    ),
  ];
  return instruction;
}

/**
 * This function can be used to unlock vested tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @returns An array of `TransactionInstruction`
 */
export async function unlock(
  connection: Connection,
  programId: PublicKey,
  seedWord: Buffer | Uint8Array,
): Promise<Array<TransactionInstruction>> {
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );
  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  const vestingTokenAccountKey = await getAssociatedTokenAddress(
    isDevnetConnection(connection)? DEVNET_TOKEN_MINT : TOKEN_MINT,
    vestingAccountKey,
    true,
  );

  const vestingInfo = await getContractInfo(connection, vestingAccountKey);

  let instruction = [
    createUnlockInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      SYSVAR_CLOCK_PUBKEY,
      vestingAccountKey,
      vestingTokenAccountKey,
      vestingInfo.destinationAddress,
      [seedWord],
    ),
  ];

  return instruction;
}

/**
 * This function can be used to initialize the unlock of vested tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param mintAddress The mint of the vested tokens
 * @returns An array of `TransactionInstruction`
 */
export async function initializeUnlock(
  connection: Connection,
  programId: PublicKey,
  seedWord: Buffer | Uint8Array,
): Promise<Array<TransactionInstruction>> {
  seedWord = seedWord.slice(0, 31);
  const [vestingAccountKey, bump] = await PublicKey.findProgramAddress(
    [seedWord],
    programId,
  );
  seedWord = Buffer.from(seedWord.toString('hex') + bump.toString(16), 'hex');

  const vestingTokenAccountKey = await getAssociatedTokenAddress(
    isDevnetConnection(connection)? DEVNET_TOKEN_MINT : TOKEN_MINT,
    vestingAccountKey,
    true,
  );

  const vestingInfo = await getContractInfo(connection, vestingAccountKey);

  let instruction = [
    createInitializeUnlockInstruction(
      programId,
      TOKEN_PROGRAM_ID,
      SYSVAR_CLOCK_PUBKEY,
      vestingAccountKey,
      vestingTokenAccountKey,
      vestingInfo.destinationAddress,
      [seedWord],
    ),
  ];

  return instruction;
}

/**
 * This function can be used retrieve information about a vesting account
 * @param connection The Solana RPC connection object
 * @param vestingAccountKey The vesting account public key
 * @returns A `ContractInfo` object
 */
export async function getContractInfo(
  connection: Connection,
  vestingAccountKey: PublicKey,
): Promise<ContractInfo> {
  console.log('Fetching contract ', vestingAccountKey.toBase58());
  const vestingInfo = await connection.getAccountInfo(
    vestingAccountKey,
    'single',
  );
  if (!vestingInfo) {
    throw new Error('Vesting contract account is unavailable');
  }
  const info = ContractInfo.fromBuffer(vestingInfo!.data);
  if (!info) {
    throw new Error('Vesting contract account is not initialized');
  }
  return info!;
}

/**
 * This function can be used to retrieve the cluster of the connection ("mainnet" or "devnet")
 * @param connection The Solana RPC connection object
 * @returns A boolean value indicating if the connection is a devnet connection
 */
export function isDevnetConnection(connection: Connection): Boolean {
  const endpoint = connection.rpcEndpoint;
  
  if (endpoint.includes('devnet')) {
    return true;
  } else return false;
}

/**
 * This function can be used to retrieve the program ID based on the connection
 * @param connection The Solana RPC connection object
 * @returns A PublicKey object representing the program ID
 */
export function getProgramId(connection: Connection): PublicKey {
  if (isDevnetConnection(connection)) {
    return DEVNET_VESTING_PROGRAM_ID;
  } else return VESTING_PROGRAM_ID;
}
