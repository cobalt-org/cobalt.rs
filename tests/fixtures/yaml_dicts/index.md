extends: index.liquid

title:   My first Blogpost
date:    24/08/2014 at 15:36
some-data:
  - nested:
      dicts: Works
  - nested:
      dicts: just fine
---
# {{ title }}
{% for entry in some-data %}
## {{ entry.nested.dicts }}
{% endfor %}