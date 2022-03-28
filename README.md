# 🧅 ROnion
Library for using the onion protocol to securely and anonymously connect to servers through onion relay nodes.

## Functionality
- Library consists of 5 main parts: 
  1. Consumer
  2. Index (Directory)
  3. Relay
  4. Protocol
  5. Cryptography
- The Consumer's main purpose is to be the client's endpoint for communication with an Index and multiple Relays
- The Index's main purpose is to manage usable Relay nodes' connection to the network. It can recieve connections from both Consumer and Relay nodes, where Consumer should receive a list of Relays the consumer can use, while the Relay nodes dotn need any responses (might change in future implementation). When a new Relay connects to Index, it is added to the Index's Relay list.
- The Relay's main purpos is to recieve payloads from an endpoint and relay that payload to another endpoint. Said endpoint can be either a Consumer, another Relay or the final unspecified endpoint indicated by the payload (e.g a webresource). 
- Onion-Protocol's main purpose is to act as this network's header and payload holder, officially known as 'Onion'. The onion indicates what Consumer, Index and Relay should do with the payload the onion holds. 
- Cryptography's main purpose is to provide ed_25519 + AES encryption of onions between Consumer, Index and Relay.

## Future work
- Finish relay nodes
- Refactor relay node code to make it more maintainable and readable.
- Write more formal documentation
- Securely handle closing of streams
- Continually update Index node to remove fallen relays

### Missing features
- Chaining multiple relays together and reliably sending data between them
- Connecting a relay to an endpoint

## Dependencies
- x25519-dalek = "1"
  - Used to gain access to diffie hellmann cryptography properties such as secrets and public keys
- ed25519-dalek = "1"
  - Used to sign and verify keys sent as plain text between nodes.  
- rand_core = { version = "0.5.1", features = ["getrandom"] }
   - Used to generate random values, used in secret and key generation
- aes = "0.8.1"
  - Used to encrypt data
- async-std = { version = "1.10.0", features = ["attributes"] }
  - Used to access asynchronous versions of the standard library

## Installation
ROnion is first and foremost a library, therefore there is no way to install it.

However, to use the library, it may be cloned and built locally using `cargo`.

Example programs for Consumer, Relay and Index may be found in the "cmd" folder.

## Usage
The ROnion library can be used to create your own versions of consumer, relays and index nodes.

It provides the necessary data structures to create onion networks and securely and anonymously sending data over it.

## Running tests
- To run this project's tests you need to use Rust's package manager `cargo`. 
- `git clone` and `cd` into the project, then run the tests using `cargo test`.

## Documentation
Documentation of the protocol may be found in [the protocol specification](PROTOCOL.md). This describes the onion data structure at the bit level and contains the specification for all flags and fields used in the communication.

Documentation for the code may be found in their corresponding source files.
