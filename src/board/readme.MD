# Board

Board is a collection of modules that handles the management and logic behind board operation. 

## A high level explanation of the Board
The board(not the module collection) is a collection of connections
to IoT Servers that is owned by one Room. This houses connections and keep track of permissions of each connection.
All actions to control a target IoT server has to go through the Board.

## It handles:
- Control over IoT server connections
- Communication with IoT server connections
- Permission checking for IoT server connections
