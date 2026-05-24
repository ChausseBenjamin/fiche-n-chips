# Fiche-N-Chips

A rewrite of the fiche terminal pastebin with extra snacks:

- single Rust binary
- dual purpose
  * terminal pastebin (netcat)
  * url-shortening (API)
- data stored in a simple sqlite database
- data compresed with brotli in the server
- web hosting (for homepage, 404, style.css robots.txt, favicon, etc...)
  * Supports either
    - files embedded in the binary
    - specified directory (file list only detected at startup)
  * Accepts multiple equivalent paths:
    - with or without trailing slashes
    - with or without the file extension
- automatic cleanup after a set time-period (1yr by default)
- privacy conscious logging (IPs can be salted+hashed for better anonimity)

**Totally useless but fun**:

Client aware responses (for errors/404s/etc...)
- Markdown/ascii versions of pages when using `cURL`
- Error messages
Example flow of a response for an inexistent URL:
  * Terminal -> Text-based message with ANSI escaped syntax highlighting
  * Terminal with no-color header -> vanilla plain-text
  * Browser -> 404.html webpage

### API examples

**Let the server pick/generate the short-url**
```sh
curl -X POST "https://1yr.ca/api/shorten" \
     -H "Content-Type: application/json" \
     -d '{"url": "https://example.com/long/url"}'
```

**Choose the short-url yourself**
```sh
curl -X POST "https://1yr.ca/api/shorten" \
     -H "Content-Type: application/json" \
     -d '{"url": "https://example.com", "slug": "mylink"}'
```

**Delete a short-url and/or a pastebin within the first 10 minutes after
its creation**

```sh
curl -X DELETE "https://1yr.ca/m3np"
```


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
