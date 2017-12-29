layout: posts.liquid

title:   My third Blogpost
published_date:    2014-08-26 15:36:20 +0100
---
# {{ page.title }}

Hey there this is my third blogpost and this is super awesome.

My Blog is lorem ipsum like, yes it is..

{% if page.previous %}
   <a class="prev" href="/{{page.previous.permalink}}">&laquo; {{page.previous.title}}</a>
 {% endif %}
 {% if page.next %}
   <a class="next" href="/{{page.next.permalink}}">{{page.next.title}} &raquo;</a>
{% endif %}
