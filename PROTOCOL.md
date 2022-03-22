# Ronion Protocol

## Onion
```
 0               1                  0                  0
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 ?? 0 1 2 3 4 5 6 7 ?? 0 1 2 3 4 5 6 7 8
+-----+-+-+-+---+---------------+--+---------------+--+-----------------+--+
|MSGT |R|C|O|TGT| Circuit ID    |..| Message len   |..| Message         |..|
| (3) |S|I|P|(2)| if CIP is set |..| (VarInt)      |..| content         |..|
|     |V|P|T|   | (VarInt)      |..|               |..|                 |..|
|     |1| |1|   |               |..|               |..|                 |..|
+-----+-+-+-+---+---------------+--+---------------+--+-----------------+--+
```

 * MSGT: Message Type (3 bits)
   0 => Hello
   1 => Close
   2 => Payload 
   3 => GetRelays
   4 => RelayPing
 * CIP: Circuit ID present.
 * RSV1..2         : Reserved for future use.
 * OPT1            : Optional bit flag.
 * TGT             : Target.
 * Circuit ID      : A Circuit ID local to a specific relay.
 * Message len     : Length of upcoming message encoded as a VarInt.
 * Message content : The message for the target.

## Message Types
 * Hello: 
   Initial **UNENCRYPTED** handshake message. The content contains the public key of the peer that is initiating the connection.
   The response content is always empty, but the Circuit ID will be set, telling the peer which Circuit ID must be used.
 
 * Close:
   Notifies peer of connection closure. The message (if any) is a UTF-8 string containing the reason for closing.
 
 * Payload: 
   A raw payload. Used for relaying data.
 
 * GetRelays: 
   Request to get all of the relays. This request must not have any message content.
   The response to this request contains all of the relays that the node is aware of.
 
 * RelayPing:
   Tells the Index node that the peer is a relay and it is still alive. This request must not have any message content.
   The response will also have an empty message content.

   
