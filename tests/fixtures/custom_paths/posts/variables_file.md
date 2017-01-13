extends: posts.liquid

title:  Variables file name
thing: hello
thang: world
date:  3 May 2015 09:05:20 +0100
path:  test/:thing/:thang.abc
---
# {{ title }}

This asserts that you can substitute any part of the url with custom variables
