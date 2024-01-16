Session - Building Social Network - II

**Requirements**

- Newly unread message indicator
- If user is offline and whenever someone sends message to a user, need to show a indicator icon on messages icon
- The indicator should only contain the number of people who sent message, not the number of message
- If user is online nothing should happen.
- The implementation will only contain 1-1 channels, no need to add complexity of groups.
- only store online users for simplicity

Websocket ref - https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs

**Client**

websocat
