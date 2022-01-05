# Auth

This is the auth handler, this collection of code handles the main logic behind authentication,

## it handles :
 - Initial user creation by using the global sql execution handler(along with the capture manager).
 - The gather of user resources via external api, for example discord user avatar and id.
 - Constructing urls for oauth.
 - Acquiring/verifying non-expired access/refresh tokens.

