# webicon

Get a website's favicon "quickly".

Inspired by
[11ty's IndieWeb Avatar](https://www.11ty.dev/docs/services/indieweb-avatar/)
service, but with the sole improvement of returning a fallback icon if nothing
is found.

## Lookup

Icons are searched for in the following order:

1. `link[rel="apple-touch-icon"]`
2. `link[rel="apple-touch-icon-precomposed"]`
3. `link[rel~="icon"]`
4. `link[rel="mask-icon"]`
5. The website origin's `/favicon.ico`

The first valid match will be returned, sized down to 64x64 if necessary.

## Self-Hosting

Because cloud hosting is expensive, and I like having money, I've set up CORS on
my deployment to only allow my website to use it (you can probably circumvent it,
but please don't). Self-hosting this should be easy enough if you just change the
constants in the rust file, and optionally provide a different `fallback.svg`.

## License

This software is licensed under Version 3 of the GNU Affero General Public
License, or at your option, any later version.
