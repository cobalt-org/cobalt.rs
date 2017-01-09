extends: posts.liquid

title:  Variables
thing: hello
date:  3 May 2015 08:05:20 +0100
path:  test/:thing/
---
# {{ title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file.
