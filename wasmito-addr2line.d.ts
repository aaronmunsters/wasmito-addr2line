/* tslint:disable */
/* eslint-disable */
export class Addr2lineError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly context: string;
}
export class Location {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly file: string | undefined;
  readonly line: number | undefined;
  readonly column: number | undefined;
}
export class Mapping {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly range_size: bigint;
  readonly file: string | undefined;
  readonly line: number | undefined;
  readonly column: number | undefined;
  readonly address: bigint;
}
export class Module {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * # Errors
   * In the case mapping fails, cf. <Error> on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  addr2line_mappings(): Mapping[];
  constructor(bytes: Uint8Array);
  /**
   * # Errors
   * In the case mapping fails, cf. <Error> on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  files(): string[];
  /**
   * # Errors
   * In the case parsing fails, cf. <Error> on retrieving the error info.
   */
  static from_wat(path: string | null | undefined, wat: string): Module;
  /**
   * # Errors
   * In the case mapping fails, cf. <Error> on retrieving the error info.
   *
   * # Note
   * Cache successive calls to this method, its result does not change.
   */
  addr2line(byte_offset: bigint): Location;
  readonly bytes: Uint8Array;
}
export class ParseError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly context: string;
}
export class StripConfig {
  free(): void;
  [Symbol.dispose](): void;
  constructor(all: boolean, to_delete: string[]);
  /**
   * # Errors
   * In the case parsing fails, cf. <Error> on retrieving the error info.
   */
  strip(module: Uint8Array): Uint8Array;
}
export class StripError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly context: string;
}
