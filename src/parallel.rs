use std::{
    io::{BufRead, Read},
    sync::mpsc::{channel, Receiver},
    thread::spawn,
};

use log::debug;

use crate::{decompress_as, CompressionFormat};

#[derive(Debug)]
pub(crate) struct ParDecompressor {
    r_decompressed: Receiver<Result<Vec<u8>, std::io::Error>>,
    buf: Vec<u8>,
    pos: usize,
}

impl ParDecompressor {
    pub(crate) fn new<B: BufRead + Send + Sync + 'static>(
        reader: B,
        format: CompressionFormat,
    ) -> Self {
        let (w_decompressed, r_decompressed) = channel();

        spawn(move || {
            let mut r = decompress_as(reader, format);
            loop {
                let read = match r.fill_buf() {
                    Ok(buf) => {
                        if buf.is_empty() {
                            debug!("Reached end of stream");
                            break;
                        } else {
                            debug!("Sending {} bytes", buf.len());
                            let res = buf.to_vec();
                            r.consume(res.len());
                            Ok(res)
                        }
                    }
                    Err(err) => Err(err),
                };
                if w_decompressed.send(read).is_err() {
                    debug!("Failed to write to channel");
                    break;
                }
            }
        });
        Self {
            r_decompressed,
            buf: Vec::new(),
            pos: 0,
        }
    }
}

impl Read for ParDecompressor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.buf.len() {
            let Ok(buf) = self.r_decompressed.recv() else {
                debug!("Receiver reached end of stream");
                return Ok(0); // End of Stream
            };
            self.buf = buf?;
            debug!("Receiver read {} bytes", self.buf.len());
            self.pos = 0;
        }
        let len = std::cmp::min(buf.len(), self.buf.len() - self.pos);
        let end = self.pos + len;
        buf[..len].copy_from_slice(&self.buf[self.pos..end]);
        self.pos += len;
        Ok(len)
    }
}
