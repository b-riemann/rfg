at the moment, for 400kb enwik.slice,
- we can obtain probcodes.u8 with entropy of 2.5915 bits/byte, leading to entropy*length = 129572.0 bytes
- using rle on probcodes.u8 -> rle.u8, the length of rle.u8 is rle_length 154155 bytes,
  - its rle_entropy is 4.3176 bits/byte, leading to rle_entropy*rle_length = 83196.6 bytes
  - huffman cosing of rle.u8 leads to a huffcodes.bin of 86753 bytes (huffcodes.tree always 256 bytes)

this is the first time i could reach better compression ratio then standard 7zip (118kb) and standard winzip (139kb) on an enwik slice.