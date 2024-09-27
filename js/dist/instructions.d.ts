/// <reference types="node" />
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { Schedule } from './state';
export declare enum Instruction {
    Init = 0,
    Create = 1
}
export declare function createInitInstruction(systemProgramId: PublicKey, vestingProgramId: PublicKey, payerKey: PublicKey, vestingAccountKey: PublicKey, seeds: Array<Buffer | Uint8Array>): TransactionInstruction;
export declare function createCreateInstruction(vestingProgramId: PublicKey, tokenProgramId: PublicKey, clockSysvarId: PublicKey, vestingAccountKey: PublicKey, vestingTokenAccountKey: PublicKey, sourceTokenAccountOwnerKey: PublicKey, sourceTokenAccountKey: PublicKey, mintAddress: PublicKey, schedule: Schedule, seeds: Array<Buffer | Uint8Array>): TransactionInstruction;
export declare function createUnlockInstruction(vestingProgramId: PublicKey, tokenProgramId: PublicKey, clockSysvarId: PublicKey, vestingAccountKey: PublicKey, vestingTokenAccountKey: PublicKey, destinationTokenAccountKey: PublicKey, seeds: Array<Buffer | Uint8Array>): TransactionInstruction;
export declare function createInitializeUnlockInstruction(vestingProgramId: PublicKey, tokenProgramId: PublicKey, clockSysvarId: PublicKey, vestingAccountKey: PublicKey, vestingTokenAccountKey: PublicKey, destinationTokenAccountKey: PublicKey, seeds: Array<Buffer | Uint8Array>): TransactionInstruction;
