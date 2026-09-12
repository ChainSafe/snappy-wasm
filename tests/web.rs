//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use snappy;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const TEST_STRING: &[u8] = b"Hello Hello Hello Hello Hello Hello ";
const TEST_STRING_COMPRESSED: &[u8] = b"$\x14Hello v\x06\x00";

#[wasm_bindgen_test]
fn test_compress() {
    let compressed = snappy::compress(TEST_STRING);
    assert_eq!(compressed.unwrap().as_ref(), TEST_STRING_COMPRESSED);
}

#[wasm_bindgen_test]
fn test_encoder() {
    let mut encoder = snappy::Encoder::new();
    assert_eq!(
        encoder.compress(TEST_STRING).unwrap().as_ref(),
        TEST_STRING_COMPRESSED
    );

    let mut compressed = vec![0; snappy::max_compress_len(TEST_STRING.len())];
    let written = encoder.compress_into(TEST_STRING, &mut compressed).unwrap();
    assert_eq!(&compressed[..written], TEST_STRING_COMPRESSED);
}

#[wasm_bindgen_test]
fn test_decompress() {
    assert_decodes(TEST_STRING_COMPRESSED, TEST_STRING);
}

#[wasm_bindgen_test]
fn test_short_extended_literals() {
    for (compressed, expected) in [
        (&b"\x01\x00A"[..], &b"A"[..]),
        (&b"\x01\xf0\x00A"[..], &b"A"[..]),
        (&b"\x02\xf0\x01AA"[..], &b"AA"[..]),
        (&b"\x01\xf4\x00\x00A"[..], &b"A"[..]),
        (&b"\x01\xf8\x00\x00\x00A"[..], &b"A"[..]),
        (&b"\x01\xfc\x00\x00\x00\x00A"[..], &b"A"[..]),
    ] {
        assert_decodes(compressed, expected);
    }
}

#[wasm_bindgen_test]
fn test_truncated_extended_literals() {
    for compressed in [
        &b"\x01\xf0"[..],
        &b"\x01\xf0\x00"[..],
        &b"\x02\xf0\x01A"[..],
        &b"\x01\xf4\x00"[..],
        &b"\x01\xf8\x00\x00"[..],
        &b"\x01\xfc\x00\x00\x00"[..],
    ] {
        assert!(snappy::decompress(compressed).is_err(), "{:x?}", compressed);
        let mut decoder = snappy::Decoder::new();
        assert!(decoder.decompress(compressed).is_err(), "{:x?}", compressed);
        let mut guarded = [0xa5; 4];
        assert!(
            decoder
                .decompress_into(compressed, &mut guarded[1..3])
                .is_err(),
            "{:x?}",
            compressed
        );
        assert_eq!(guarded[0], 0xa5);
        assert_eq!(guarded[3], 0xa5);
        assert_eq!(decoder.decompress(b"\x01\x00A").unwrap().as_ref(), b"A");
    }
}

#[wasm_bindgen_test]
fn test_length_prefixes() {
    for compressed in [
        &b"\x01\x00A"[..],
        &b"\x81\x00\x00A"[..],
        &b"\x81\x80\x00\x00A"[..],
        &b"\x81\x80\x80\x00\x00A"[..],
        &b"\x81\x80\x80\x80\x00\x00A"[..],
    ] {
        assert_decodes(compressed, b"A");
    }
    for compressed in [
        &b"\x80\x80\x80\x80\x80\x00"[..],
        &b"\x81\x80\x80\x80\x80\x80\x80\x80\x80\x00\x00A"[..],
    ] {
        assert!(snappy::decompress_len(compressed).is_err());
        assert!(snappy::decompress(compressed).is_err());
        let mut decoder = snappy::Decoder::new();
        assert!(decoder.decompress(compressed).is_err());
        assert!(decoder.decompress_into(compressed, &mut [0; 1]).is_err());
    }
}

#[wasm_bindgen_test]
fn test_undersized_output() {
    let mut decoder = snappy::Decoder::new();
    let mut guarded = [0xa5; 3];
    assert!(decoder
        .decompress_into(b"\x02\x04AB", &mut guarded[1..2])
        .is_err());
    assert_eq!(guarded[0], 0xa5);
    assert_eq!(guarded[2], 0xa5);
}

fn assert_decodes(compressed: &[u8], expected: &[u8]) {
    assert_eq!(snappy::decompress_len(compressed).unwrap(), expected.len());
    assert_eq!(
        snappy::decompress(compressed).unwrap().as_ref(),
        expected,
        "{:x?}",
        compressed
    );
    let mut decoder = snappy::Decoder::new();
    assert_eq!(
        decoder.decompress(compressed).unwrap().as_ref(),
        expected,
        "{:x?}",
        compressed
    );
    let end = expected.len() + 1;
    let mut guarded = vec![0xa5; end + 1];
    assert_eq!(
        decoder
            .decompress_into(compressed, &mut guarded[1..end])
            .unwrap(),
        expected.len()
    );
    assert_eq!(&guarded[1..end], expected);
    assert_eq!(guarded[0], 0xa5);
    assert_eq!(guarded[end], 0xa5);
}
