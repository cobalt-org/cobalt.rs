extends: posts.liquid

title:  Variables
thing: hello
path:  test/:thing/
---
# {{ title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file.
