NYTT:
Onionkryptering mellom hvert relay.
Når relay sender respons til forrige relay, skal denne krypteres med klientens secret (som før), men i tillegg med onion som også inneholder sin egen circuit id som forrige relay bruker for å sende videre. 


## Kobling til entry node (relay 0)
Consumer sender HelloRequest(secret_public_0) til Entry_node (relay_node_0)

Consumer mottar HelloResponse(entry_long) fra Entry_node.

- Konklusjon: Consumer har mottat entry_long.

## Kobling til relay 1
Consumer sender HelloRequest som onion, (kryptert med entry_cipher) til Entry_node

Entry_node mottar og dekrypterer onion. Får ut target(Relay_node_1), circuit_id=None og HelloRequest(secret_public_1). 

Entry_node sjekker om han har en kryptert kanal til Relay_node_1. 
FALSE?
-> Entry_node sender HelloRequest(local_secret_1) til Relay_node_1.
-> Relay_node mottar HelloResponse(local_1_long) fra Relay_node_1.

Entry_node krypterer target=Current, circuit_id=UID og HelloRequest(secret_public_1) til en onion med local_1_cipher og sender til Relay_node_1.

Rela_node_1 mottar og dekrypterer onion til en target(current), circuit_id og HelloRequest(secret_public_1).

Relay_node_1 svarer med HelloResponse(relay_1_long), kryptert til onion for Entry_node (bruker local_entry_cipher -which is the same as local_1_cipher). 

Entry_node mottar onion, dekrypterer med local_1_cipher og får ut target(), cicuit_id og HelloResponse(relay_1_long) fra Relay_node_1.

Entry_node sender HelloResponse(relay_1_long) som onion (kryptert med entry_cipher) til Consumer.

Consumer mottar onion og dekrypterer med entry_cipher. Får ut target, circuit_id og HelloResponse(relay_1_long). 

- Konklusjon: Consumer har mottat relay_1_long.