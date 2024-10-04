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
  mintAddress!: PublicKey;
  isInitialized!: boolean;

  constructor(
    destinationAddress: PublicKey,
    mintAddress: PublicKey,
    isInitialized: boolean,
  ) {
    this.destinationAddress = destinationAddress;
    this.mintAddress = mintAddress;
    this.isInitialized = isInitialized;
  }

  static fromBuffer(buf: Buffer): VestingScheduleHeader {
    const destinationAddress = new PublicKey(buf.slice(0, 32));
    const mintAddress = new PublicKey(buf.slice(32, 64));
    const isInitialized = buf[64] == 1;
    const header: VestingScheduleHeader = {
      destinationAddress,
      mintAddress,
      isInitialized,
    };
    return header;
  }
}

export class ContractInfo {
  destinationAddress!: PublicKey;
  mintAddress!: PublicKey;
  schedule!: Schedule;

  constructor(
    destinationAddress: PublicKey,
    mintAddress: PublicKey,
    schedule: Schedule,
  ) {
    this.destinationAddress = destinationAddress;
    this.mintAddress = mintAddress;
    this.schedule = schedule;
  }

  static fromBuffer(buf: Buffer): ContractInfo | undefined {
    const header = VestingScheduleHeader.fromBuffer(buf.slice(0, 65));
    if (!header.isInitialized) {
      return undefined;
    }
    const schedule = Schedule.fromBuffer(buf.slice(65, 81));
    return new ContractInfo(
      header.destinationAddress,
      header.mintAddress,
      schedule,
    );
  }
}
