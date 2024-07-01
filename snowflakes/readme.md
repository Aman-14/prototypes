### Snowflake

Snowflake ID is a 64-bit unique identifier that consists of three parts: timestamp, worker ID, and sequence number.
The timestamp is a 41-bit integer that represents the number of milliseconds since a certain epoch time.

The worker ID is a 10-bit integer that identifies the worker generating the ID, and the sequence number is a 12-bit integer that ensures uniqueness in case multiple IDs are generated within the same millisecond by the same worker.

```
0                                       41     51         64
+---------------------------------------+------+-----------+
| timestamp (milliseconds since epoch)  |worker| sequence  |
+---------------------------------------+------+-----------+
```
