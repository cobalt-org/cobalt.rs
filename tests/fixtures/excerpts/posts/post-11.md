layout: default.liquid

title: Reference-style links immediately below first block do not resolve
published_date: 2017-02-11 04:51:34 -0000
---

References are placed immediately below the first paragraph:
[30][] [31][] [32][] [33][]
[30]: /0
 [31]: /1
  [32]: /2
   [33]: /3

Note that this is at it should be, as the markdown implementations I've tried
(including CommonMark) do not render the above links (when the post is rendered
individually).
