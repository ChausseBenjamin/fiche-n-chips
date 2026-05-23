# Fiche-N-Chips

A rewrite of the fiche terminal pastebin with extra snacks

- single Rust binary
- dual purpose
    - terminal pastebin (netcat)
    - url-shortening (API)
- data stored in a simple sqlite database
- data compresed with brotli in the server
- web hosting (for homepage, 404, style.css robots.txt, etc...) with either
    - files pre-compiled in the single binary
    - sim

## Motivations

[Fiche][1] is a beautiful piece of software I became aware of through
[termbin.com][2] and used extensively. At some point, termbin failed for a
few days which led me to look into self-hosting it in a container. I was not
impressed with how deploying the stack required multiple container (nginx
for files, python for the API, fiche to ingest data can create server-side
files).

I had already impulse-bought the [1yr.ca][3] domain a bit ago when I realized
no-one had it yet. The concept was perfect: *Fiche, but for a whole year*.

And so I decided to build the infra and add the extra features I wanted along
the way.











[1]: https://github.com/solusipse/fiche
[2]: https://termbin.com
[3]: https://1yr.ca
