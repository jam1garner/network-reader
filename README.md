# network-reader

A client/server protocol for using `io::Read` and `io::Seek` over a network

**Server example:**
```rust
use network_reader::Networked;

Networked::new_buffered(File::open("my_file.txt").unwrap(), ("127.0.0.1", 4000))
    .unwrap()
    .listen()
    .unwrap();
```

**Client example:**
```rust
use network_reader::NetworkReader;

let mut reader = NetworkReader::new(("127.0.0.1", 4000)).unwrap();

// Read 4 bytes from Reader provided over the network
let mut buf = [0u8; 4];
reader.read_exact(&mut buf).unwrap();
```

If the two above samples were used together, the client would read the first 4 bytes from file "my_file.txt".
