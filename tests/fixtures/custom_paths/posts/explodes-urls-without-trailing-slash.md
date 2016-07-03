extends: posts.liquid

title:  Boom without trailing slash
path:  test/thing4
---
# {{ title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file, even when the user did not specify a trailing slash.
