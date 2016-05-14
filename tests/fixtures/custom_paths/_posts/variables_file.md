extends: posts.liquid

title:  Variables file name
thing: hello
thang: world
path:  test/:thing/:thang.abc
---
# {{ title }}

This asserts that you can substitute any part of the url with custom variables
