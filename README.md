# 🧅 ROnion
Library for using the onion protocol to securely and anonymously connect to servers through onion relay nodes.

## Functionality
- Library consists of 5 main parts: Consumer, Index, Relay, Onion-Protocol and Cryptography.
- The Consumer's main purpose is to be the client's endpoint for communication with an Index and multiple Relays
- The Index's main purpos is to manage usable Relay nodes' connection to the network. It can recieve connections from both Consumer and Relay nodes, where Consumer should recieve a list of Relays the consumer can use, while the Relay nodes dotn need any responses (might change in future implementation). When a new Relay connects to Index, it is added to the Index's Relay list.
- The Relay's main purpos is to recieve payloads from an endpoint and relay that payload to another endpoint. Said endpoint can be either a Consumer, another Relay or the final unspecified endpoint indicated by the payload (e.g a webresource). 
- Onion-Protocol's main purpos is to act as this network's header and payload holder, officially known as 'Onion'. The onion indicates what Consumer, Index and Relay should do with the payload the onion holds. 
- Cryptography's main purpos is to provide ed_25519 + AES encryption of onions between Consumer, Index and Relay.
## Future work

### Missing features
- .
### Potential weaknesses
- .

## Dependencies
- .

## Installation

## Usage

## Running tests
- To run this project's tests you need to use Rust's package manager 'cargo'. 
- Clone and cd into the project, then run the tests with the command:$ cargo test

## Documentation
