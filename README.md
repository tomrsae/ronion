# 🧅 ROnion
Library for using the onion protocol to securely and anonymously connect to servers through onion relay nodes.

## Functionality
- Library consists of 5 main parts: 
  1. Consumer
  2. Relay
  3. Protocol
  4. Cryptography
- The Consumer's main purpose is to be the client's endpoint for communication with an Index and multiple Relays
- The Index's main purpos is to manage usable Relay nodes' connection to the network. It can recieve connections from both Consumer and Relay nodes, where Consumer should recieve a list of Relays the consumer can use, while the Relay nodes dotn need any responses (might change in future implementation). When a new Relay connects to Index, it is added to the Index's Relay list.
- The Relay's main purpos is to recieve payloads from an endpoint and relay that payload to another endpoint. Said endpoint can be either a Consumer, another Relay or the final unspecified endpoint indicated by the payload (e.g a webresource). 
- Onion-Protocol's main purpos is to act as this network's header and payload holder, officially known as 'Onion'. The onion indicates what Consumer, Index and Relay should do with the payload the onion holds. 
- Cryptography's main purpos is to provide ed_25519 + AES encryption of onions between Consumer, Index and Relay.
## Future work
- Refactor and 
### Missing features
- .
### Potential weaknesses


## Dependencies
- x25519-dalek = "1"
- ed25519-dalek = "1"
- rand_core = { version = "0.5.1", features = ["getrandom"] }
- aes = "0.8.1"
- async-std = { version = "1.10.0", features = ["attributes"] }

## Installation
- Can not install the program, as it is mainly a library.

## Usage
- Library can be used to create own versions of consumer, relays and index nodes. 

## Running tests
- To run this project's tests you need to use Rust's package manager 'cargo'. 
- Clone and cd into the project, then run the tests with the command:$ cargo test

## Documentation
