# naive compression experiments using enwik9

This is a testbed for me learning stuff about compression techniques, motivated by enwik9, although only compression ratios down to zip-like tools are approximately reached with far too large time and memory complexity.

current sizes for the first 1_000_000 bytes (files without exe):

| encoding Path                         | files    |    size |
| ------------------------------------- | -------- | ------- |
| prepd(v2b)-probcodes-rle-huffencode16 | tree,bin | 303_896 |
| prepd(v2)-probcodes-rle-huffencode16  | tree,bin | 303_897 |
| prepd(v2)-probcodes-huffencode8       |          | 315_739 |
| prepd(v0)-probcodes-huffencode8       |          | 319_217 |
| prepd(v0)-probcodes-huffencode16      |          | 319_990 |