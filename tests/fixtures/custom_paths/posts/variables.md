layout: posts.liquid

title:  Variables
published_date:  3 May 2015 08:05:20 +0100
permalink: /test/:thing/
data:
  thing: hello
---
# {{ title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file.
