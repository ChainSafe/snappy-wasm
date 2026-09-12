const assert = require('node:assert/strict');
const snappy = require('../pkg/snappy.js');

const valid = [
  ['empty', '00', ''],
  ['canonical literal', '010041', '41'],
  ['one-byte length, one-byte literal', '01f00041', '41'],
  ['one-byte length, two-byte literal', '02f0014142', '4142'],
  ['two-byte length, one-byte literal', '01f4000041', '41'],
  ['three-byte length', '01f800000041', '41'],
  ['four-byte length', '01fc0000000041', '41'],
  ['five-byte header', '81808080000041', '41'],
];

const invalidHeaders = [
  ['six-byte header', '8180808080000041'],
  ['ten-byte header', '818080808080808080000041'],
];

const malformed = [
  ['missing one-byte length', '01f0'],
  ['truncated two-byte length', '01f400'],
  ['truncated three-byte length', '01f80000'],
  ['truncated four-byte length', '01fc000000'],
  ['missing literal', '01f000'],
  ['truncated literal', '02f00141'],
  ['truncated copy', '05004101'],
  ['underfilled output', '020041'],
  ['overfilled output', '01044142'],
  ...invalidHeaders,
];

function inputs(bytes) {
  const padded = new Uint8Array(bytes.length + 8).fill(0xa5);
  padded.set(bytes, 4);
  return [Buffer.from(bytes), padded.subarray(4, -4)];
}

function assertBytes(actual, expected, label) {
  assert.ok(actual instanceof Uint8Array, label);
  assert.deepEqual(actual, Uint8Array.from(expected), label);
}

const decoder = new snappy.Decoder();
const encoder = new snappy.Encoder();
const retained = [];

function decodeInto(input, length) {
  const padded = new Uint8Array(length + 8).fill(0xa5);
  const output = padded.subarray(4, -4);
  try {
    const written = decoder.decompress_into(input, output);
    assert.equal(written, length);
    return output.subarray(0, written);
  } finally {
    assert.deepEqual(padded.subarray(0, 4), new Uint8Array(4).fill(0xa5));
    assert.deepEqual(padded.subarray(-4), new Uint8Array(4).fill(0xa5));
  }
}

const methods = [
  ['decompress', (input) => snappy.decompress(input)],
  ['Decoder.decompress', (input) => decoder.decompress(input)],
  ['Decoder.decompress_into', decodeInto],
];

try {
  for (const [name, raw, decoded] of valid) {
    const expected = Buffer.from(decoded, 'hex');
    for (const input of inputs(Buffer.from(raw, 'hex'))) {
      const original = Uint8Array.from(input);
      assert.equal(snappy.decompress_len(input), expected.length, name);
      for (const [method, decode] of methods) {
        const label = `${method}: ${name}`;
        const output = decode(input, expected.length);
        assertBytes(output, expected, label);
        assert.deepEqual(Uint8Array.from(input), original, label);
        retained.push([output, Uint8Array.from(expected), label]);
      }
      input.fill(0);
    }
  }

  for (const [name, raw] of malformed) {
    for (const input of inputs(Buffer.from(raw, 'hex'))) {
      const original = Uint8Array.from(input);
      for (const [method, decode] of methods) {
        assert.throws(
          () => decode(input, 8),
          (error) => error instanceof Error && !(error instanceof assert.AssertionError),
          `${method}: ${name}`
        );
        assert.deepEqual(Uint8Array.from(input), original, name);
        assertBytes(decoder.decompress(Buffer.from('010041', 'hex')), [0x41], 'reuse after error');
      }
    }
  }

  for (const [name, raw] of invalidHeaders) {
    for (const input of inputs(Buffer.from(raw, 'hex'))) {
      assert.throws(() => snappy.decompress_len(input), Error, name);
    }
  }

  for (const input of inputs(Buffer.from('02f0014142', 'hex'))) {
    assert.throws(
      () => decodeInto(input, 1),
      (error) => error instanceof Error && !(error instanceof assert.AssertionError),
      'undersized output'
    );
    assertBytes(decodeInto(input, 2), [0x41, 0x42], 'reuse after undersized output');
  }

  for (const source of [new Uint8Array(), Buffer.from('Hello '.repeat(20)), new Uint8Array(65536).fill(42)]) {
    for (const input of inputs(source)) {
      const original = Uint8Array.from(input);
      const compressed = [snappy.compress(input), encoder.compress(input)];
      const target = new Uint8Array(snappy.max_compress_len(input.length));
      const written = encoder.compress_into(input, target);
      compressed.push(target.subarray(0, written));
      for (const bytes of compressed) {
        assertBytes(snappy.decompress(bytes), original, 'static roundtrip');
        assertBytes(decoder.decompress(bytes), original, 'reusable roundtrip');
        assertBytes(decodeInto(bytes, original.length), original, 'into roundtrip');
      }
      assert.deepEqual(Uint8Array.from(input), original, 'compression preserves input');
    }
  }

  for (const [output, expected, label] of retained) {
    assertBytes(output, expected, `retained output: ${label}`);
  }
} finally {
  decoder.free();
  encoder.free();
}

console.log('Node package tests passed');
