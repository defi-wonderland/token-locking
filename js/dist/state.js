import { PublicKey } from '@solana/web3.js';
import { Numberu64 } from './utils';
export class Schedule {
    constructor(timeDelta, amount) {
        this.timeDelta = timeDelta;
        this.amount = amount;
    }
    toBuffer() {
        return Buffer.concat([this.timeDelta.toBuffer(), this.amount.toBuffer()]);
    }
    static fromBuffer(buf) {
        const timeDelta = Numberu64.fromBuffer(buf.slice(0, 8));
        const amount = Numberu64.fromBuffer(buf.slice(8, 16));
        return new Schedule(timeDelta, amount);
    }
}
export class VestingScheduleHeader {
    constructor(destinationAddress, mintAddress, isInitialized) {
        this.destinationAddress = destinationAddress;
        this.mintAddress = mintAddress;
        this.isInitialized = isInitialized;
    }
    static fromBuffer(buf) {
        const destinationAddress = new PublicKey(buf.slice(0, 32));
        const mintAddress = new PublicKey(buf.slice(32, 64));
        const isInitialized = buf[64] == 1;
        const header = {
            destinationAddress,
            mintAddress,
            isInitialized,
        };
        return header;
    }
}
export class ContractInfo {
    constructor(destinationAddress, mintAddress, schedules) {
        this.destinationAddress = destinationAddress;
        this.mintAddress = mintAddress;
        this.schedules = schedules;
    }
    static fromBuffer(buf) {
        const header = VestingScheduleHeader.fromBuffer(buf.slice(0, 65));
        if (!header.isInitialized) {
            return undefined;
        }
        const schedules = [];
        for (let i = 65; i < buf.length; i += 16) {
            schedules.push(Schedule.fromBuffer(buf.slice(i, i + 16)));
        }
        return new ContractInfo(header.destinationAddress, header.mintAddress, schedules);
    }
}
