import {
  PublicKey,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { CreateSchedule } from './state';

export enum Instruction {
  Init,
  Create,
}

export function createInitInstruction(
  systemProgramId: PublicKey,
  vestingProgramId: PublicKey,
  payerKey: PublicKey,
  vestingAccountKey: PublicKey,
  seeds: Array<Buffer | Uint8Array>,
): TransactionInstruction {
  let buffers = [Buffer.from(Int8Array.from([0]).buffer), Buffer.concat(seeds)];

  const data = Buffer.concat(buffers);
  const keys = [
    {
      pubkey: systemProgramId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: SYSVAR_RENT_PUBKEY,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: payerKey,
      isSigner: true,
      isWritable: true,
    },
    {
      pubkey: vestingAccountKey,
      isSigner: false,
      isWritable: true,
    },
  ];

  return new TransactionInstruction({
    keys,
    programId: vestingProgramId,
    data,
  });
}

export function createCreateInstruction(
  vestingProgramId: PublicKey,
  tokenProgramId: PublicKey,
  clockSysvarId: PublicKey,
  vestingAccountKey: PublicKey,
  vestingTokenAccountKey: PublicKey,
  sourceTokenAccountOwnerKey: PublicKey,
  sourceTokenAccountKey: PublicKey,
  schedule: CreateSchedule,
  seeds: Array<Buffer | Uint8Array>,
): TransactionInstruction {
  let buffers = [Buffer.from(Int8Array.from([1]).buffer), Buffer.concat(seeds)];

  buffers.push(schedule.toBuffer());

  const data = Buffer.concat(buffers);
  const keys = [
    {
      pubkey: tokenProgramId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: clockSysvarId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: vestingAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: vestingTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: sourceTokenAccountOwnerKey,
      isSigner: true,
      isWritable: false,
    },
    {
      pubkey: sourceTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
  ];
  return new TransactionInstruction({
    keys,
    programId: vestingProgramId,
    data,
  });
}

export function createUnlockInstruction(
  vestingProgramId: PublicKey,
  tokenProgramId: PublicKey,
  clockSysvarId: PublicKey,
  vestingAccountKey: PublicKey,
  vestingTokenAccountKey: PublicKey,
  destinationTokenAccountKey: PublicKey,
  seeds: Array<Buffer | Uint8Array>,
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from(Int8Array.from([2]).buffer),
    Buffer.concat(seeds),
  ]);

  const keys = [
    {
      pubkey: tokenProgramId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: clockSysvarId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: vestingAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: vestingTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: destinationTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
  ];
  return new TransactionInstruction({
    keys,
    programId: vestingProgramId,
    data,
  });
}

export function createInitializeUnlockInstruction(
  vestingProgramId: PublicKey,
  tokenProgramId: PublicKey,
  clockSysvarId: PublicKey,
  vestingAccountKey: PublicKey,
  vestingTokenAccountKey: PublicKey,
  destinationTokenAccountKey: PublicKey,
  seeds: Array<Buffer | Uint8Array>,
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from(Int8Array.from([3]).buffer),
    Buffer.concat(seeds),
  ]);

  const keys = [
    {
      pubkey: tokenProgramId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: clockSysvarId,
      isSigner: false,
      isWritable: false,
    },
    {
      pubkey: vestingAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: vestingTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
    {
      pubkey: destinationTokenAccountKey,
      isSigner: false,
      isWritable: true,
    },
  ];
  return new TransactionInstruction({
    keys,
    programId: vestingProgramId,
    data,
  });
}
