import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type Result = { 'ok' : bigint } |
  { 'err' : string };
export interface _SERVICE {
  'executeDualSwap' : ActorMethod<[bigint, boolean], Result>,
  'getReserves' : ActorMethod<[], [bigint, bigint]>,
}
