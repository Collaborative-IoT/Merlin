/*
The communication handler tests aims at
testing the logic that the handler enforces.

It doesn't test its consuming modules like the 
execution handler, it makes sure
requests fail under certain circumstances,
state is being modified and messages are being
persisted to the voice server via RabbitMq.

This test isn't integration based, so we manually
grab the messages intended for the voice server after
publish and assert them.

*/

pub async fn test(){

}