/// <reference types="node" />
import { PublicKey, TransactionInstruction, Connection } from '@solana/web3.js';
import { ContractInfo, Schedule } from './state';
/**
 * The vesting schedule program ID on mainnet
 */
export declare const TOKEN_VESTING_PROGRAM_ID: PublicKey;
/**
 * This function can be used to lock tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param payer The fee payer of the transaction
 * @param sourceTokenOwner The owner of the source token account (i.e where locked tokens are originating from)
 * @param possibleSourceTokenPubkey The source token account (i.e where locked tokens are originating from), if null it defaults to the ATA
 * @param destinationTokenPubkey The destination token account i.e where unlocked tokens will be transfered
 * @param mintAddress The mint of the tokens being vested
 * @param schedule The vesting schedule
 * @returns An array of `TransactionInstruction`
 */
export declare function create(connection: Connection, programId: PublicKey, seedWord: Buffer | Uint8Array, payer: PublicKey, sourceTokenOwner: PublicKey, possibleSourceTokenPubkey: PublicKey | null, destinationTokenPubkey: PublicKey, mintAddress: PublicKey, schedule: Schedule): Promise<Array<TransactionInstruction>>;
/**
 * This function can be used to unlock vested tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param mintAddress The mint of the vested tokens
 * @returns An array of `TransactionInstruction`
 */
export declare function unlock(connection: Connection, programId: PublicKey, seedWord: Buffer | Uint8Array, mintAddress: PublicKey): Promise<Array<TransactionInstruction>>;
/**
 * This function can be used to initialize the unlock of vested tokens
 * @param connection The Solana RPC connection object
 * @param programId The token vesting program ID
 * @param seedWord Seed words used to derive the vesting account
 * @param mintAddress The mint of the vested tokens
 * @returns An array of `TransactionInstruction`
 */
export declare function initializeUnlock(connection: Connection, programId: PublicKey, seedWord: Buffer | Uint8Array, mintAddress: PublicKey): Promise<Array<TransactionInstruction>>;
/**
 * This function can be used retrieve information about a vesting account
 * @param connection The Solana RPC connection object
 * @param vestingAccountKey The vesting account public key
 * @returns A `ContractInfo` object
 */
export declare function getContractInfo(connection: Connection, vestingAccountKey: PublicKey): Promise<ContractInfo>;
