extends: default.tpl
---
This is my Index page!

{% for post in posts %}
 <a href="_posts/{{post.name}}.html">{{ post.title }}</a>
{% endfor %}
