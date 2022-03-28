# Ronion Protocol

## Onion
```

+---------------+--------------------------+--+---------------+--+---------------+--+-------------------+--+
|0 1 2 3 4 5 6 7| 4 BYTES / 16 BYTES       |  | VARINT        |  | VARINT        |  | Message len BYTES |  |
+-----+-+-+-+---+--------------------------+--+---------------+--+---------------+--+-------------------+--+
|MSGT |R|C|O|TGT| if TGT = IP AND OPT1 = 0 |..| Circuit ID    |..| Message len   |..| Message           |..|
| (3) |S|I|P|(2)| IPv4 octets (32)         |..| if CIP is set |..| (VarInt)      |..| content           |..|
|     |V|P|T|   +------------------------- |..|               |..|               |..|                   |..|
|     |1| |1|   | if TGT = IP AND OPT1 = 1 |..|               |..|               |..|                   |..|
|     | | | |   | IPv6 octets (128)        |..|               |..|               |..|                   |..|
+-----+-+-+-+---+--------------------------+--+---------------+--+---------------+--+-------------------+--+
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
 * TGT             : Target (0 = Relay, 1 = IP, 2 = Current)
 * Circuit ID      : A Circuit ID local to a specific relay.
 * Message len     : Length of upcoming message encoded as a VarInt.
 * Message content : The message for the target.

## Message Types
 * HelloRequest: 
   **UNENCRYPTED** Handshake message. The content contains the signing public key of the peer that is initiating the connection, as well as it's type (Consumer or Relay).

 * HelloResponse:
   **UNENCRYPTED** Handshake response. The content contains the signed diffie hellman public key.
 
 * Close:
   Notifies peer of connection closure. The message (if any) is a UTF-8 string containing the reason for closing.
 
 * Payload: 
   A raw payload. Used for relaying data.
 
 * GetRelaysRequest: 
   Request to get all of the relays. This request must not have any message content.

 * GetRelaysResponse:
   The response to this request contains all of the relays that the index is aware of.
 
 * RelayPingRequest:
   Tells the Index that the peer is a relay and it is still alive. The content contains a port number and a public signing key.

* RelayPingResponse:
   The response will also have an empty message content.

   
