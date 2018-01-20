layout: posts.liquid

title:  Variables file name
published_date:  2015-05-03 09:05:20 +0100
permalink: /test/{{data.thing}}/{{data.thang}}.abc
data:
  thing: hello
  thang: world
---
# {{ page.title }}

This asserts that you can substitute any part of the url with custom variables
