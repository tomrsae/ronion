
## Kobling til entry node (relay 0)
Consumer sender HelloRequest(secret_public_0) til Entry_node (relay_node_0)

Consumer mottar HelloResponse(entry_long) fra Entry_node.

- Konklusjon: Consumer har mottat entry_long.

## Koblign til relay 1
Consumer sender HelloRequest som onion, (kryptert med entry_cipher) til Entry_node

Entry_node mottar og dekrypterer onion. Får ut target, circuit_id og HelloRequest(secret_public_1). 

Entry_node viederesender HelloRequest(secret_public_1) til Relay_node_1.

Entry_node mottar HelloResponse(relay_1_long) fra Relay_node_1.

Entry_node sender HelloResponse(relay_1_long) som onion (kryptert med entry_cipher) til Consumer.

Consumer mottar onion og dekrypterer med entry_cipher. Får ut target, circuit_id og HelloResponse(relay_1_long). 

- Konklusjon: Consumer har mottat relay_1_long.

## Kobling til relay 2
Consumer krypterer HelloRequest(secret_public_2) som en onion med relay_1_cipher.

Consumer krypterer Payload(onion) som en onion med entry_cipher og sender til Entry_node

Entry_node dekrypterer onion og får ut target, circuit_id og Payload(onion).

Entry_node sender onion som lå i Payload til Relay_node_1.

Relay_node_1 mottar og dekrypterer onion. Får ut target, circuit_id og HelloRequest(secret_public_2). 

Relay_node_1 sender HelloRequest(secret_public_2) til Relay_node_2.

Relay_node_1 mottar HelloResponse(relay_2_long) fra Relay_node_2.

Relay_node_1 sender HelloResponse(relay_2_long) som onion (kryptert med relay_1_cipher) til Entry_node.

Entry_node mottar onion fra relay_node_1, krypterer den med entry_cipher og sender til Consumer.

Consumer mottar onion fra entry_node, dekrypterer først med entry_cipher og deretter med relay_1_cipher. 
Får ut target, circuit_id og HelloResponse(relay_2_long). 

- Konklusjon: Consumer har mottat relay_2_long.


Slik fortsetter det i mønster for n antall relays...