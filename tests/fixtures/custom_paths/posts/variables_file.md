layout: posts.liquid

title:  Variables file name
published_date:  3 May 2015 09:05:20 +0100
permalink: /test/{{data.thing}}/{{data.thang}}.abc
data:
  thing: hello
  thang: world
---
# {{ title }}

This asserts that you can substitute any part of the url with custom variables
