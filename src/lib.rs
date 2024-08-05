use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Encoder(snap::raw::Encoder);

#[wasm_bindgen]
impl Encoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Encoder(snap::raw::Encoder::new())
    }

    pub fn compress(&mut self, input: &[u8]) -> Result<Box<[u8]>, JsError> {
        match self.0.compress_vec(input) {
            Ok(output) => Ok(output.into_boxed_slice()),
            Err(err) => Err(handle_err(err)),
        }
    }

    pub fn compress_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, JsError> {
        match self.0.compress(input, output) {
            Ok(len) => Ok(len),
            Err(err) => Err(handle_err(err)),
        }
    }
}

#[wasm_bindgen]
pub struct Decoder(snap::raw::Decoder);

#[wasm_bindgen]
impl Decoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Decoder(snap::raw::Decoder::new())
    }

    pub fn decompress(&mut self, input: &[u8]) -> Result<Box<[u8]>, JsError> {
        match self.0.decompress_vec(input) {
            Ok(output) => Ok(output.into_boxed_slice()),
            Err(err) => Err(handle_err(err)),
        }
    }

    pub fn decompress_into(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize, JsError> {
        match self.0.decompress(input, output) {
            Ok(len) => Ok(len),
            Err(err) => Err(handle_err(err)),
        }
    }
}

#[wasm_bindgen]
pub fn compress(input: &[u8]) -> Result<Box<[u8]>, JsError> {
    let mut encoder = snap::raw::Encoder::new();
    match encoder.compress_vec(input) {
        Ok(output) => Ok(output.into_boxed_slice()),
        Err(err) => Err(handle_err(err)),
    }
}

#[wasm_bindgen]
pub fn decompress(input: &[u8]) -> Result<Box<[u8]>, JsError> {
    let mut decoder = snap::raw::Decoder::new();
    match decoder.decompress_vec(input) {
        Ok(output) => Ok(output.into_boxed_slice()),
        Err(err) => Err(handle_err(err)),
    }
}

#[wasm_bindgen]
/// Returns the maximum compressed size given the uncompressed size.
///
/// If the uncompressed size exceeds the maximum allowable size then this
/// returns 0.
pub fn max_compress_len(input_len: usize) -> usize {
    snap::raw::max_compress_len(input_len)
}

#[wasm_bindgen]
/// Returns the decompressed size (in bytes) of the compressed bytes given.
///
/// `input` must be a sequence of bytes returned by a conforming Snappy
/// compressor.
///
/// # Errors
///
/// This function returns an error in the following circumstances:
///
/// * An invalid Snappy header was seen.
/// * The total space required for decompression exceeds `2^32 - 1`.
pub fn decompress_len(input: &[u8]) -> Result<usize, JsError> {
    match snap::raw::decompress_len(input) {
        Ok(len) => Ok(len),
        Err(err) => Err(handle_err(err)),
    }
}

fn handle_err(err: snap::Error) -> JsError {
    JsError::new(err.to_string().as_str())
}
