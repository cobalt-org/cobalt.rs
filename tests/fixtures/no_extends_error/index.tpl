extends: default.tpl
---
This is my Index page!

{% for post in posts %}
 {{ post.title }}
{% endfor %}
