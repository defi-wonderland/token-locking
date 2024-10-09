import { PublicKey } from '@solana/web3.js';
import { Numberu64 } from './utils';

export class CreateSchedule {
  // Release time in unix timestamp
  timeDelta!: Numberu64;
  amount!: Numberu64;

  constructor(timeDelta: Numberu64, amount: Numberu64) {
    this.timeDelta = timeDelta;
    this.amount = amount;
  }

  public toBuffer(): Buffer {
    return Buffer.concat([this.timeDelta.toBuffer(), this.amount.toBuffer()]);
  }

  static fromBuffer(buf: Buffer): Schedule {
    const timeDelta: Numberu64 = Numberu64.fromBuffer(buf.slice(0, 8));
    const amount: Numberu64 = Numberu64.fromBuffer(buf.slice(8, 16));
    return new Schedule(timeDelta, amount);
  }
}

export class Schedule {
  // Release time in unix timestamp
  releaseDate!: Numberu64;
  amount!: Numberu64;

  constructor(releaseDate: Numberu64, amount: Numberu64) {
    this.releaseDate = releaseDate;
    this.amount = amount;
  }

  public toBuffer(): Buffer {
    return Buffer.concat([this.releaseDate.toBuffer(), this.amount.toBuffer()]);
  }

  static fromBuffer(buf: Buffer): Schedule {
    const releaseDate: Numberu64 = Numberu64.fromBuffer(buf.slice(0, 8));
    const amount: Numberu64 = Numberu64.fromBuffer(buf.slice(8, 16));
    return new Schedule(releaseDate, amount);
  }
}

export class VestingScheduleHeader {
  destinationAddress!: PublicKey;
  isInitialized!: boolean;

  constructor(destinationAddress: PublicKey, isInitialized: boolean) {
    this.destinationAddress = destinationAddress;
    this.isInitialized = isInitialized;
  }

  static fromBuffer(buf: Buffer): VestingScheduleHeader {
    const destinationAddress = new PublicKey(buf.slice(0, 32));
    const isInitialized = buf[32] == 1;
    const header: VestingScheduleHeader = {
      destinationAddress,
      isInitialized,
    };
    return header;
  }
}

export class ContractInfo {
  destinationAddress!: PublicKey;
  schedule!: Schedule;

  constructor(destinationAddress: PublicKey, schedule: Schedule) {
    this.destinationAddress = destinationAddress;
    this.schedule = schedule;
  }

  static fromBuffer(buf: Buffer): ContractInfo | undefined {
    const header = VestingScheduleHeader.fromBuffer(buf.slice(0, 33));
    if (!header.isInitialized) {
      return undefined;
    }
    const schedule = Schedule.fromBuffer(buf.slice(33, 49));
    return new ContractInfo(header.destinationAddress, schedule);
  }
}
