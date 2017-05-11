extends: posts.liquid

title:   My second Blogpost
date:    25/08/2014 at 15:36
---
# {{ title }}

Hey there this is my second blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..

{% if previous %}
   <a class="prev" href="/{{previous.path}}">&laquo; {{previous.title}}</a>
 {% endif %}
 {% if next %}
   <a class="next" href="/{{next.path}}">{{next.title}} &raquo;</a>
{% endif %}