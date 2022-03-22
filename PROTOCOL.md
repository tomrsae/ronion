# Ronion Protocol

## Onion
0                1                  
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 ?? 0 1 2 3 4 5 6 7 ?? 0 1 2 3 4 5 6 7 8
+-----+-+-+-+---+---------------+--+---------------+--+-----------------+--+
|MSGT |R|C|O|TGT| Circuit ID    |..| Message len   |..| Message         |..|
| (3) |S|I|P|(2)| if CIP is set |..| (VarInt)      |..|                 |..|
|     |V|P|T|   | (VarInt)      |..|               |..|                 |..|
|     |1| |1|   |               |..|               |..|                 |..|
+-----+-+-+-+---+---------------+--+---------------+--+-----------------+--+

 * MSGT: Message Type (3 bits)
   0 => Hello
   1 => Close
   2 => Payload 
   3 => GetRelays
   4 => RelayPing
 * CIP: Circuit ID present.
 * RSV1..2     : Reserved for future use.
 * OPT1        : Optional bit flag.
 * TGT         : Target.
 * Circuit ID  : A Circuit ID local to a specific relay.
 * Message len : Length of upcoming message encoded as a VarInt.
 * Messge      : The message for the target.


