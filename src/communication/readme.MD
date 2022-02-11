# Communication

Communication is a collection of modules that handle direct communication between server and client. When a user sends a request the communication module
executes that request and sometimes generates a response based on the success of that action assuming it is a request that doesn't have to first pass through the voice server. Some requests require the voice server to execute first and then we will get the voice server's response, then continue. Things like changing your death/mute status won't require the voice server's involvement and you will get a direct response from the communication handler. To see responses that are dictated by the voice server's response, go check the vs_response module. 

The bottom line: Every request starts here, and may end here based on whether or not it requires the voice server's input. 


For example, if a user decides to change their username, the communication
module is tasked with making sure it isn't a duplicate, executing the sql for storage and giving a response to the user.

# It handles:
- Opcode request routing
- Response generation and sending
- Data fetching
- Data capturing
- Making sure users aren't doing illegal things, like kicking users if they don't have permission.
