extends: posts.liquid

title:  Boom without trailing slash
date:  3 May 2015 05:05:20 +0100
path:  test/thing4
---
# {{ title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file, even when the user did not specify a trailing slash.
