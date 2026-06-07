// Generates a 1024x1024 solid-teal PNG with zero dependencies (Node zlib only),
// used as the source for `tauri icon`. Color = Echo slate-teal #0b3b3c.
import { writeFileSync } from "node:fs";
import { deflateSync } from "node:zlib";
import { fileURLToPath } from "node:url";

const SIZE = 1024;
const [R, G, B] = [11, 59, 60];

function crc32(buf) {
  let c = ~0 >>> 0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (~c) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 2; // color type: truecolor RGB
// compression=0, filter=0, interlace=0 already zeroed

const row = Buffer.alloc(1 + SIZE * 3); // leading filter byte (0) per scanline
for (let x = 0; x < SIZE; x++) {
  row[1 + x * 3] = R;
  row[2 + x * 3] = G;
  row[3 + x * 3] = B;
}
const raw = Buffer.concat(Array.from({ length: SIZE }, () => row));
const idat = deflateSync(raw);

const png = Buffer.concat([
  sig,
  chunk("IHDR", ihdr),
  chunk("IDAT", idat),
  chunk("IEND", Buffer.alloc(0)),
]);

const out = fileURLToPath(new URL("../icon-src.png", import.meta.url));
writeFileSync(out, png);
console.log(`wrote ${out} (${png.length} bytes)`);
