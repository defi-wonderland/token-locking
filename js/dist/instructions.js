import { SYSVAR_RENT_PUBKEY, TransactionInstruction, } from '@solana/web3.js';
export var Instruction;
(function (Instruction) {
    Instruction[Instruction["Init"] = 0] = "Init";
    Instruction[Instruction["Create"] = 1] = "Create";
})(Instruction || (Instruction = {}));
export function createInitInstruction(systemProgramId, vestingProgramId, payerKey, vestingAccountKey, seeds) {
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
export function createCreateInstruction(vestingProgramId, tokenProgramId, clockSysvarId, vestingAccountKey, vestingTokenAccountKey, sourceTokenAccountOwnerKey, sourceTokenAccountKey, destinationTokenAccountKey, mintAddress, schedule, seeds) {
    let buffers = [
        Buffer.from(Int8Array.from([1]).buffer),
        Buffer.concat(seeds),
        mintAddress.toBuffer(),
        destinationTokenAccountKey.toBuffer(),
    ];
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
export function createUnlockInstruction(vestingProgramId, tokenProgramId, clockSysvarId, vestingAccountKey, vestingTokenAccountKey, destinationTokenAccountKey, seeds) {
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
export function createInitializeUnlockInstruction(vestingProgramId, tokenProgramId, clockSysvarId, vestingAccountKey, vestingTokenAccountKey, destinationTokenAccountKey, seeds) {
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
