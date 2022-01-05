# Communication

Communication is a collection of modules that handle direct communication between server and client. When a user sends a request the communication module
executes that request and generates a response based on the success of that action. For example, if a user decides to change their username, the communication
module is tasked with making sure it isn't a duplicate, executing the sql for storage and giving a response to the user.

# It handles:
- Opcode request routing
- Response generation and sending
- Data fetching
- Data capturing
- Making sure users aren't doing illegal things, like kicking users if they don't have permission.
