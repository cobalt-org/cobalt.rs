layout: posts.liquid

title:  Variables
published_date:  2015-05-03 08:05:20 +0100
permalink: /test/{{data.thing}}/
data:
  thing: hello
---
# {{ page.title }}

This asserts that custom paths without a file extension get made into a folder with an index.html file.
