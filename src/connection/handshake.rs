pub enum Handshake {
    /// The expected message to neeed from a client in order to ACK their connection request
    Syn,

    /// The server's acknowldgment of a connection from a peer
    Ack
}
