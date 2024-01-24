# Bloom Filter in Rust

## Overview

A Bloom Filter is a space-efficient probabilistic data structure used to test whether an element is a member of a set.
It may return false positives but never false negatives. Bloom filters are particularly useful in scenarios where memory is limited, and quick membership tests are required.

## Implementation

### `BloomFilter` Struct

The `BloomFilter` struct is designed to represent a Bloom Filter with a fixed-size bit array.
The size of the filter is determined during its creation.
The implementation includes methods to add elements to the filter (`add`), check for existence (`exists`).

### `hash` Function

The `hash` function utilizes the MurmurHash3 algorithm to generate a hash for a given key. This hash is then used to determine the index within the Bloom Filter's bit array.
