/// <reference types="node" />
import { PublicKey } from '@solana/web3.js';
import { Numberu64 } from './utils';
export declare class Schedule {
    timeDelta: Numberu64;
    amount: Numberu64;
    constructor(timeDelta: Numberu64, amount: Numberu64);
    toBuffer(): Buffer;
    static fromBuffer(buf: Buffer): Schedule;
}
export declare class VestingScheduleHeader {
    destinationAddress: PublicKey;
    mintAddress: PublicKey;
    isInitialized: boolean;
    constructor(destinationAddress: PublicKey, mintAddress: PublicKey, isInitialized: boolean);
    static fromBuffer(buf: Buffer): VestingScheduleHeader;
}
export declare class ContractInfo {
    destinationAddress: PublicKey;
    mintAddress: PublicKey;
    schedules: Array<Schedule>;
    constructor(destinationAddress: PublicKey, mintAddress: PublicKey, schedules: Array<Schedule>);
    static fromBuffer(buf: Buffer): ContractInfo | undefined;
}
