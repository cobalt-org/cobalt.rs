layout: posts.liquid

title:   My first Blogpost
published_date:    24 Aug 2014 15:36:20 +0100
---
# {{ title }}

Hey there this is my first blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..

{% if previous %}
   <a class="prev" href="/{{previous.path}}">&laquo; {{previous.title}}</a>
 {% endif %}
 {% if next %}
   <a class="next" href="/{{next.path}}">{{next.title}} &raquo;</a>
{% endif %}
