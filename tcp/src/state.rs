
#[derive(Debug)]
pub enum State {
    // represents waiting for a connection request 
    // from any remote TCP peer and port
    Listen,

    // represents waiting for a matching connection 
    // request after having sent a connection request.
    SynSent,

    // represents waiting for a confirming connection 
    // request acknowledgment after having both received 
    // and sent a connection request
    SynReceived,
    
    // represents an open connection, data received can be 
    // delivered to the user. The normal state for the data 
    // transfer phase of the connection.
    Established,

    // represents waiting for a connection termination request 
    // from the remote TCP peer, or an acknowledgment of the 
    // connection termination request previously sent.
    FinWait1,

    // represents waiting for a connection termination request 
    // from the remote TCP peer.
    FinWait2,

    // represents waiting for a connection termination request 
    // from the local user.
    CloseWait,

    // represents waiting for a connection termination request 
    // acknowledgment from the remote TCP peer.
    Closing,

    // represents waiting for an acknowledgment of the connection 
    // termination request previously sent to the remote TCP peer 
    // (this termination request sent to the remote TCP peer already 
    // included an acknowledgment of the termination request sent from the remote TCP peer).
    LastAck,

    // represents waiting for enough time to pass to be sure the remote TCP 
    // peer received the acknowledgment of its connection termination request 
    // and to avoid new connections being impacted by delayed segments from previous connections.
    TimeWait,

    // represents no connection state at all.
    Closed,
}