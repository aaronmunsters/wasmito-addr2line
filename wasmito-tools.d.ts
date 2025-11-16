/* tslint:disable */
/* eslint-disable */
export class Addr2lineError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  context(): string;
}
export class Location {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  file(): string | undefined;
  line(): number | undefined;
  column(): number | undefined;
}
export class Mapping {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  range_size(): bigint;
  file(): string | undefined;
  line(): number | undefined;
  column(): number | undefined;
  address(): bigint;
}
export class MappingIncludingOffset {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  range_size(): bigint;
  instructions(): PositionedInstruction[];
  file(): string | undefined;
  line(): number | undefined;
  column(): number | undefined;
  address(): bigint;
}
export class Module {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * # Errors
   * In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  addr2line_mappings(): Mapping[];
  /**
   * # Errors
   * In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  addr2line_mappings_with_offsets(): MappingIncludingOffset[];
  constructor(bytes: Uint8Array);
  bytes(): Uint8Array;
  /**
   * # Errors
   * In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  files(): string[];
  /**
   * # Errors
   * In the case parsing fails, cf. [`ParseError`] on retrieving the error info.
   */
  static from_wat(path: string | null | undefined, wat: string): Module;
  /**
   * # Errors
   * In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  addr2line(byte_offset: bigint): Location;
}
export class ParseError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  context(): string;
}
export class PositionedInstruction {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  instr(): string;
  address(): number;
}
export class StripConfig {
  free(): void;
  [Symbol.dispose](): void;
  constructor(all: boolean, to_delete: string[]);
  /**
   * # Errors
   * In the case parsing fails, cf. [`StripError`] on retrieving the error info.
   */
  strip(module: Uint8Array): Uint8Array;
}
export class StripError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly context: string;
}
