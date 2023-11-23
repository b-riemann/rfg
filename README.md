at the moment, for 400kb enwik.slice,
- we can obtain probcodes.u8 with entropy of 2.5915 bits/byte, leading to entropy*length = 129572.0 bytes
- using rle on probcodes.u8 -> rle.u8, the length of rle.u8 is rle_length 154155 bytes,
  and its rle_entropy is 4.3176 bits/byte, leading to rle_entropy*rle_length = 83196.6 bytes

this is the first time i could hope for better copression then standard 7zip (118kb) and standard winzip (139kb)