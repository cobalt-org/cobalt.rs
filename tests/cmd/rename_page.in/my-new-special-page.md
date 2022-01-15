---
title: My New Special Page
layout: default.liquid
---
## Blog!

{% for post in collections.posts.pages %}
#### {{post.title}}

[{{ post.title }}]({{ post.permalink }})
{% endfor %}
