# Using CDC for compression

Imagine that [`words.txt`](words.txt) represents the final of a series of versions of a file. Each version was obtained by inserting one of the lines which is currently present in the file. (The words are meaningless and were drawn from `/usr/share/dict/words` and ROT13'd to reduce the risk of anything inappropriate. There are probably no duplicate words.)

This repository seeks to discover how effectively we can store the collection of all versions of the file in a deduplicated manner by using [content-defined chunking](https://en.wikipedia.org/wiki/Content-Defined_Chunking). This uses a rolling hash to split each version of the file into variably-sized chunks, such that sufficiently long common regions of the file (regardless of how they are aligned) will be split into the exact same chunks which can then be deduplicated using interning.

With 100 lines, the results looks like this:

![Desmos graph showing three curves representing cumulative byte counts for the first N versions. The data for the resulting curve can be found in `report.txt`.](image/growth.png).

- The black dots (top) represent the number of bytes needed to store all versions without compression. Since every new version stores the previous version along with an additional line, it has quadratic growth.
- The green dots (bottom) represent the bytes needed to store only the latest version, which grows linearly.
- The purple dots (middle) represent the bytes needed to store all versions using CDC deduplication. It is calculated as `(total bytes of all distinct chunks) + (4 * # of references to chunks)`, with the theory that the model could be implemented using having each version refer to a series of chunks by 32-bit hash.

By the end of 100 versions, storing with CDC compression still ends up about 7-8x overhead compared to just storing the last version. This is too high for most people to accept for the use case of, e.g., backing up full version history of a file. The graphs would be closer if they were doing larger inserts. There are a few reasons for the overhead:

- There's a high constant factor to inserts, because e.g. adding a 20-byte line to a file will also disrupt the chunking of the regions around it, creating at least 3-4 unique chunks which are each ~64 bytes.
- The growth rate seems to be superlinear, **p ≈ 1.32**. I will need to look closer to understand why this occurs.

The natural thing to compare CDC to is traditional data compression for eliminating redundancy across many similar versions of a file. This is what compressed file systems like Squashfs and archivers like rzip do. A few things to consider with this comparison:

- Although I haven't tried it, I would expect any dictionary compression algorithm to get far lower overhead and a closer-to-linear growth rate (with a sufficiently long dictionary/context window, at least) on this example, because it can take better advantage of fine-grained redundancy.
- Both CDC compression and decompression can be straightforwardly parallelized with minimal impact on compression quality. CDC decoding can also use vectored I/O.
- CDC can probably be as fast as dictionary compression or faster with a large chunk size, since the amount of work done by the rolling hash per byte is very low, and the chunk hash function can be SIMD. Compression speed doesn't really grow with the size of the dictionary.
- CDC decoding can be random access (as long as you store some index that helps you find the right place in the reference list) with no impact on compression quality.
- It's possible to use hierarchical chunking (where the lists of chunk references are themselves broken into chunks) to get compression ratios better than `(size of a chunk reference) / (average size of a chunk)`.

A few other notes about this CDC implementation:

- I'm using a very simple polynomial rolling hash function, which is getting a strangely high number of collisions. Using a better rolling hash, or using some of the newer CDC approaches which don't exactly rely on a rolling hash, might fix this.
- Unlike other applications of rolling hashes, there isn't really a downside in CDC to using as small of a window size as possible. In the extreme case, we can imagine that if the input files consisted of uniformly random bytes you could get effective chunking by splitting on every occurrence of an arbitrary byte (if the input files were code, it might be useful to choose this byte to be a newline). The only reason to use a window size of more than a few bytes is to ensure you get enough distinct hashes to reduce the risk of pathological/malicious inputs causing bad chunking.
- This implementation doesn't set a minimum or maximum chuk size, which is similarly needed to avoid pathological behavior but probably also reduces the quality of chunking.
